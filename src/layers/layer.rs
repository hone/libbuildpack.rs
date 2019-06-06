use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use toml;

use super::{
    config::Config,
    env::{EnvSet, Envs},
};

const BUILD_ENV_FOLDER: &str = "env.build";
const LAUNCH_ENV_FOLDER: &str = "env.launch";
const SHARED_ENV_FOLDER: &str = "env";

pub struct Layer {
    // path to the root directory for the layer
    root: PathBuf,
    name: String,
    pub config: Config,
    pub envs: Envs,
}

impl Layer {
    pub fn new(root: &str, name: &str) -> Result<Self, std::io::Error> {
        let root_path = PathBuf::from(&root);
        let layer_path = root_path.join(&name);
        std::fs::create_dir_all(&layer_path)?;

        Ok(Layer {
            root: root_path,
            name: name.to_string(),
            envs: Envs::new(),
            config: Config::new(),
        })
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join(format!("{}.toml", &self.name))
    }

    pub fn layer_path(&self) -> PathBuf {
        self.root.join(&self.name)
    }

    pub fn profile_d_path(&self) -> PathBuf {
        self.layer_path().join("profile.d")
    }

    pub fn write_metadata(&self) -> Result<(), std::io::Error> {
        let string = toml::to_string(&self.config).unwrap();
        let mut file = File::create(&self.config_path())?;
        file.write_all(string.as_bytes())?;

        Ok(())
    }

    pub fn read_metadata(&mut self) -> Result<(), std::io::Error> {
        let mut file = match File::open(&self.config_path()) {
            Ok(file) => file,
            Err(_) => return Ok(()),
        };
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.config = toml::from_str(&contents).unwrap();

        Ok(())
    }

    pub fn remove_metadata(&self) -> Result<(), std::io::Error> {
        if self.config_path().is_file() {
            std::fs::remove_file(&self.config_path())?;
        }

        Ok(())
    }

    pub fn write_profile_d(&self, name: &str, contents: &str) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.profile_d_path())?;
        let file_path = &self.profile_d_path().join(name);
        let mut file = File::create(&file_path)?;
        file.write_all(contents.as_bytes())?;

        Ok(())
    }

    pub fn write_envs(&self) -> Result<(), std::io::Error> {
        self.write_env(BUILD_ENV_FOLDER, &self.envs.build)
            .and(self.write_env(SHARED_ENV_FOLDER, &self.envs.shared))
            .and(self.write_env(LAUNCH_ENV_FOLDER, &self.envs.launch))
    }

    fn write_env(&self, folder: &str, env: &EnvSet) -> Result<(), std::io::Error> {
        let folder_path = &self.layer_path().join(folder);
        std::fs::create_dir_all(&folder_path)?;

        for (key, value) in env.append_path.vars() {
            let mut file = File::create(&folder_path.join(key))?;
            file.write_all(value.as_bytes())?;
        }

        for (key, value) in env.append.vars() {
            let filename = format!("{}.append", key);
            let mut file = File::create(&folder_path.join(filename))?;
            file.write_all(value.as_bytes())?;
        }

        for (key, value) in env.r#override.vars() {
            let filename = format!("{}.override", key);
            let mut file = File::create(&folder_path.join(filename))?;
            file.write_all(value.as_bytes())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use toml::value::Value;

    struct Setup {
        tmp_dir: TempDir,
        pub root_path: PathBuf,
        pub name: String,
        pub layer: Layer,
    }

    fn setup() -> Setup {
        let tmp_dir = TempDir::new("libbuildpack.rs").unwrap();
        let root_path = tmp_dir.path().join("layers");
        let name = "foo".to_owned();
        let layer = Layer::new(root_path.to_str().unwrap(), &name).unwrap();

        Setup {
            root_path: root_path,
            tmp_dir: tmp_dir,
            name: name,
            layer: layer,
        }
    }

    fn test_env_file(env_file: &PathBuf, value: &str) {
        let mut contents = String::new();
        let mut file = File::open(&env_file).unwrap();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, value);
    }

    #[test]
    fn it_creates_a_new_layer() {
        let setup = setup();

        assert!(&setup.root_path.join(&setup.name).exists());
    }

    #[test]
    fn it_writes_metdata() {
        let mut setup = setup();
        let layer = &mut setup.layer;
        let toml_file = &setup.root_path.join("foo.toml");

        layer.config.launch = Some(true);
        layer.config.build = Some(false);

        let config = &mut layer.config;
        let metadata = config.metadata_as_mut();
        metadata.insert("foo".to_string(), Value::String("bar".to_string()));
        assert!(layer.write_metadata().is_ok());
        assert!(toml_file.exists());

        let mut file = File::open(&toml_file).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mut config: Config = toml::from_str(&contents).unwrap();
        assert_eq!(config.launch, Some(true));
        assert_eq!(config.build, Some(false));
        assert!(config.cache.is_none());
        assert_eq!(
            config.metadata_as_mut().get("foo").unwrap().as_str(),
            Some("bar")
        );
    }

    #[test]
    fn it_reads_metadata() {
        let mut setup = setup();

        let mut file = File::create(&setup.root_path.join("foo.toml")).unwrap();
        writeln!(
            file,
            r#"
launch = true
build = false

[metadata]
foo = "bar"
"#
        )
        .unwrap();

        let layer = &mut setup.layer;
        let read = layer.read_metadata();
        assert!(read.is_ok());
        assert_eq!(layer.config.launch.unwrap(), true);
        assert_eq!(layer.config.build.unwrap(), false);
        assert!(layer.config.cache.is_none());
        assert_eq!(
            layer.config.metadata_as_mut().get("foo").unwrap().as_str(),
            Some("bar")
        );
    }

    #[test]
    fn it_reads_metadata_when_no_file() {
        let mut setup = setup();
        let layer = &mut setup.layer;
        let read = layer.read_metadata();
        assert!(read.is_ok());
    }

    #[test]
    fn it_can_set_build_env_vars() {
        let mut setup = setup();
        let mut layer = &mut setup.layer;
        let mut build_env = &mut layer.envs.build;
        let env_folder = setup.root_path.join(setup.name).join("env.build");

        build_env.append_path.set_var("FOO", "foo");
        build_env.append.set_var("BAR", "bar");
        build_env.r#override.set_var("BAZ", "baz");
        assert!(layer.write_envs().is_ok());
        test_env_file(&env_folder.join("FOO"), "foo");
        test_env_file(&env_folder.join("BAR.append"), "bar");
        test_env_file(&env_folder.join("BAZ.override"), "baz");
    }

    #[test]
    fn it_can_set_launch_env_vars() {
        let mut setup = setup();
        let mut layer = &mut setup.layer;
        let mut launch_env = &mut layer.envs.launch;
        let env_folder = setup.root_path.join(setup.name).join("env.launch");

        launch_env.append_path.set_var("FOO", "foo");
        launch_env.append.set_var("BAR", "bar");
        launch_env.r#override.set_var("BAZ", "baz");
        assert!(layer.write_envs().is_ok());
        test_env_file(&env_folder.join("FOO"), "foo");
        test_env_file(&env_folder.join("BAR.append"), "bar");
        test_env_file(&env_folder.join("BAZ.override"), "baz");
    }

    #[test]
    fn it_can_set_shared_env_vars() {
        let mut setup = setup();
        let mut layer = &mut setup.layer;
        let mut shared_env = &mut layer.envs.shared;
        let env_folder = setup.root_path.join(setup.name).join("env");

        shared_env.append_path.set_var("FOO", "foo");
        shared_env.append.set_var("BAR", "bar");
        shared_env.r#override.set_var("BAZ", "baz");
        assert!(layer.write_envs().is_ok());
        test_env_file(&env_folder.join("FOO"), "foo");
        test_env_file(&env_folder.join("BAR.append"), "bar");
        test_env_file(&env_folder.join("BAZ.override"), "baz");
    }

    #[test]
    fn it_removes_metadata_that_does_not_exist() {
        let mut setup = setup();
        let mut layer = &mut setup.layer;

        assert!(layer.remove_metadata().is_ok());
    }

    #[test]
    fn it_removes_metadata_that_does_exist() {
        let mut setup = setup();
        let layer = &setup.layer;
        let metadata_path = setup.root_path.join(format!("{}.toml", &setup.name));
        let mut file = File::create(&metadata_path).unwrap();
        file.write_all(
            "
build = false
cache = false
launch = true

[metadata]
version = 1
"
            .as_bytes(),
        )
        .unwrap();

        assert!(layer.remove_metadata().is_ok());
        assert!(!metadata_path.is_file());
    }

    #[test]
    fn it_writes_profile_d() {
        let setup = setup();
        let layer = &setup.layer;
        let profile_d_path = setup.root_path.join("foo").join("profile.d").join("foo.sh");

        assert!(layer.write_profile_d("foo.sh", "exit 0").is_ok());
        test_env_file(&profile_d_path, "exit 0");
    }
}
