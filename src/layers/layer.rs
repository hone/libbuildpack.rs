use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use crate::error::Result;

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
    pub fn new(root: &str, name: &str) -> Result<Self> {
        let root_path = PathBuf::from(&root);
        let layer_path = root_path.join(&name);
        fs::create_dir_all(&layer_path)?;

        Ok(Layer {
            root: root_path,
            name: name.to_string(),
            envs: Envs::new(),
            config: Config::new(),
        })
    }

    pub fn config<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Config),
    {
        f(&mut self.config);
        self.write_metadata()
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

    pub fn write_metadata(&self) -> Result<()> {
        let string = toml::to_string(&self.config)?;
        fs::write(&self.config_path(), string)?;

        Ok(())
    }

    pub fn read_metadata(&mut self) -> Result<()> {
        let path = self.config_path();

        if path.exists() {
            let contents = fs::read_to_string(path)?;
            self.config = toml::from_str(&contents)?;
        }

        Ok(())
    }

    pub fn remove_metadata(&self) -> Result<()> {
        if self.config_path().is_file() {
            fs::remove_file(&self.config_path())?;
        }

        Ok(())
    }

    pub fn write_profile_d(&self, name: &str, contents: &str) -> Result<()> {
        fs::create_dir_all(&self.profile_d_path())?;
        let file_path = &self.profile_d_path().join(name);
        fs::write(&file_path, contents)?;

        Ok(())
    }

    pub fn write_envs(&self) -> Result<()> {
        Self::write_env(&self.layer_path(), BUILD_ENV_FOLDER, &self.envs.build)
            .and(Self::write_env(
                &self.layer_path(),
                SHARED_ENV_FOLDER,
                &self.envs.shared,
            ))
            .and(Self::write_env(
                &self.layer_path(),
                LAUNCH_ENV_FOLDER,
                &self.envs.launch,
            ))
    }

    fn write_env(layer_path: &PathBuf, folder: &str, env: &EnvSet) -> Result<()> {
        let folder_path = layer_path.join(folder);
        fs::create_dir_all(&folder_path)?;

        for (key, value) in env.append_path.vars() {
            fs::write(&folder_path.join(key), value)?;
        }

        for (key, value) in env.append.vars() {
            let filename = format!("{}.append", key);
            fs::write(&folder_path.join(filename), value)?;
        }

        for (key, value) in env.r#override.vars() {
            let filename = format!("{}.override", key);
            fs::write(&folder_path.join(filename), value)?;
        }

        Ok(())
    }

    pub fn read_envs(&mut self) -> Result<()> {
        Self::read_env(&self.layer_path(), BUILD_ENV_FOLDER, &mut self.envs.build)
            .and(Self::read_env(
                &self.layer_path(),
                LAUNCH_ENV_FOLDER,
                &mut self.envs.launch,
            ))
            .and(Self::read_env(
                &self.layer_path(),
                SHARED_ENV_FOLDER,
                &mut self.envs.shared,
            ))
    }

    fn read_env(layer: &PathBuf, folder: &str, env: &mut EnvSet) -> Result<()> {
        env.clear();

        let folder_path = layer.join(folder);
        if !folder_path.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(folder_path)? {
            let entry = entry?;
            let env_path = entry.path();
            let value = fs::read_to_string(entry.path())?;

            let ext = env_path.extension().unwrap_or(OsStr::new(""));
            let mut key_path = entry.path();
            if ext == "append" || ext == "override" {
                key_path.set_extension("");
            }

            // returns `None` if path terminates in `..`
            if let Some(key) = key_path.file_name() {
                if ext == "append" {
                    env.append.set_var(key, value);
                } else if ext == "override" {
                    env.r#override.set_var(key, value);
                } else {
                    env.append_path.set_var(key, value);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;
    use std::result::Result;
    use tempdir::TempDir;
    use toml::value::Value;

    struct Setup {
        _tmp_dir: TempDir,
        pub root_path: PathBuf,
        pub name: String,
        pub layer: Layer,
    }

    fn setup() -> Result<Setup, Error> {
        let tmp_dir = TempDir::new("libbuildpack.rs")?;
        let root_path = tmp_dir.path().join("layers");
        let name = "foo".to_owned();
        let layer = Layer::new(root_path.to_str().unwrap(), &name)?;

        Ok(Setup {
            root_path: root_path,
            _tmp_dir: tmp_dir,
            name: name,
            layer: layer,
        })
    }

    fn test_env_file(env_file: &PathBuf, value: &str) -> Result<(), Error> {
        let contents = fs::read_to_string(&env_file)?;

        assert_eq!(contents, value);

        Ok(())
    }

    #[test]
    fn it_creates_a_new_layer() -> Result<(), Error> {
        let setup = setup()?;

        assert!(&setup.root_path.join(&setup.name).exists());

        Ok(())
    }

    #[test]
    fn it_can_set_config() -> Result<(), Error> {
        let setup = setup()?;
        let mut layer = setup.layer;

        assert!(layer
            .config(|c| {
                c.launch = Some(true);
                c.build = Some(true);
                c.cache = Some(false);
            })
            .is_ok());

        assert_eq!(Some(true), layer.config.launch);
        assert_eq!(Some(true), layer.config.build);
        assert_eq!(Some(false), layer.config.cache);
        assert!(&setup.root_path.join("foo.toml").exists());

        Ok(())
    }

    #[test]
    fn it_writes_metdata() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let toml_file = &setup.root_path.join("foo.toml");

        layer.config.launch = Some(true);
        layer.config.build = Some(false);

        let metadata = layer.config.metadata_as_mut();
        metadata.insert("foo".to_string(), Value::String("bar".to_string()));
        assert!(layer.write_metadata().is_ok());
        assert!(toml_file.exists());

        let contents = fs::read_to_string(&toml_file)?;
        let mut config: Config = toml::from_str(&contents)?;
        assert_eq!(config.launch, Some(true));
        assert_eq!(config.build, Some(false));
        assert!(config.cache.is_none());
        assert_eq!(
            config.metadata_as_mut().get("foo").unwrap().as_str(),
            Some("bar")
        );

        Ok(())
    }

    #[test]
    fn it_reads_metadata() -> Result<(), Error> {
        let mut setup = setup()?;

        fs::write(
            &setup.root_path.join("foo.toml"),
            r#"
launch = true
build = false

[metadata]
foo = "bar"
"#,
        )?;

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

        Ok(())
    }

    #[test]
    fn it_reads_metadata_when_no_file() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let read = layer.read_metadata();
        assert!(read.is_ok());

        Ok(())
    }

    #[test]
    fn it_can_set_build_env_vars() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let build_env = &mut layer.envs.build;
        let env_folder = setup.root_path.join(setup.name).join("env.build");

        build_env.append_path.set_var("FOO", "foo");
        build_env.append.set_var("BAR", "bar");
        build_env.r#override.set_var("BAZ", "baz");
        assert!(layer.write_envs().is_ok());
        test_env_file(&env_folder.join("FOO"), "foo")?;
        test_env_file(&env_folder.join("BAR.append"), "bar")?;
        test_env_file(&env_folder.join("BAZ.override"), "baz")?;

        Ok(())
    }

    #[test]
    fn it_can_set_launch_env_vars() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let launch_env = &mut layer.envs.launch;
        let env_folder = setup.root_path.join(setup.name).join("env.launch");

        launch_env.append_path.set_var("FOO", "foo");
        launch_env.append.set_var("BAR", "bar");
        launch_env.r#override.set_var("BAZ", "baz");
        assert!(layer.write_envs().is_ok());
        test_env_file(&env_folder.join("FOO"), "foo")?;
        test_env_file(&env_folder.join("BAR.append"), "bar")?;
        test_env_file(&env_folder.join("BAZ.override"), "baz")?;

        Ok(())
    }

    #[test]
    fn it_can_set_shared_env_vars() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let shared_env = &mut layer.envs.shared;
        let env_folder = setup.root_path.join(setup.name).join("env");

        shared_env.append_path.set_var("FOO", "foo");
        shared_env.append.set_var("BAR", "bar");
        shared_env.r#override.set_var("BAZ", "baz");
        assert!(layer.write_envs().is_ok());
        test_env_file(&env_folder.join("FOO"), "foo")?;
        test_env_file(&env_folder.join("BAR.append"), "bar")?;
        test_env_file(&env_folder.join("BAZ.override"), "baz")?;

        Ok(())
    }

    #[test]
    fn it_can_read_build_env_vars() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let env_folder = setup.root_path.join(setup.name).join("env.build");

        fs::create_dir_all(&env_folder)?;
        fs::write(env_folder.join("FOO"), "foo")?;
        fs::write(env_folder.join("BAR.append"), "bar")?;
        fs::write(env_folder.join("BAZ.override"), "baz")?;

        assert!(layer.read_envs().is_ok());

        assert_eq!(
            layer.envs.build.append_path.var("FOO"),
            Ok("foo".to_string())
        );
        assert_eq!(layer.envs.build.append.var("BAR"), Ok("bar".to_string()));
        assert_eq!(
            layer.envs.build.r#override.var("BAZ"),
            Ok("baz".to_string())
        );

        Ok(())
    }

    #[test]
    fn it_can_read_launch_env_vars() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let env_folder = setup.root_path.join(setup.name).join("env.launch");

        fs::create_dir_all(&env_folder)?;
        fs::write(env_folder.join("FOO"), "foo")?;
        fs::write(env_folder.join("BAR.append"), "bar")?;
        fs::write(env_folder.join("BAZ.override"), "baz")?;

        assert!(layer.read_envs().is_ok());

        assert_eq!(
            layer.envs.launch.append_path.var("FOO"),
            Ok("foo".to_string())
        );
        assert_eq!(layer.envs.launch.append.var("BAR"), Ok("bar".to_string()));
        assert_eq!(
            layer.envs.launch.r#override.var("BAZ"),
            Ok("baz".to_string())
        );

        Ok(())
    }

    #[test]
    fn it_can_read_shared_env_vars() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;
        let env_folder = setup.root_path.join(setup.name).join("env");

        fs::create_dir_all(&env_folder)?;
        fs::write(env_folder.join("FOO"), "foo")?;
        fs::write(env_folder.join("BAR.append"), "bar")?;
        fs::write(env_folder.join("BAZ.override"), "baz")?;

        assert!(layer.read_envs().is_ok());

        assert_eq!(
            layer.envs.shared.append_path.var("FOO"),
            Ok("foo".to_string())
        );
        assert_eq!(layer.envs.shared.append.var("BAR"), Ok("bar".to_string()));
        assert_eq!(
            layer.envs.shared.r#override.var("BAZ"),
            Ok("baz".to_string())
        );

        Ok(())
    }

    #[test]
    fn it_removes_metadata_that_does_not_exist() -> Result<(), Error> {
        let mut setup = setup()?;
        let layer = &mut setup.layer;

        assert!(layer.remove_metadata().is_ok());

        Ok(())
    }

    #[test]
    fn it_removes_metadata_that_does_exist() -> Result<(), Error> {
        let setup = setup()?;
        let layer = &setup.layer;
        let metadata_path = setup.root_path.join(format!("{}.toml", &setup.name));
        fs::write(
            &metadata_path,
            "
build = false
cache = false
launch = true

[metadata]
version = 1
",
        )?;

        assert!(layer.remove_metadata().is_ok());
        assert!(!metadata_path.is_file());

        Ok(())
    }

    #[test]
    fn it_writes_profile_d() -> Result<(), Error> {
        let setup = setup()?;
        let layer = &setup.layer;
        let profile_d_path = setup.root_path.join("foo").join("profile.d").join("foo.sh");

        assert!(layer.write_profile_d("foo.sh", "exit 0").is_ok());
        test_env_file(&profile_d_path, "exit 0")?;

        Ok(())
    }
}
