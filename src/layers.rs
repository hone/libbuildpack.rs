mod config;
mod env;
mod launch;
mod layer;
use launch::Launch;
pub use layer::Layer;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const ROOT_LAYER_FOLDER: &str = "/layers";
const LAUNCH_TOML_FILE: &str = "launch.toml";

pub struct Layers {
    root: PathBuf,
    pub launch: Launch,
}

impl Layers {
    pub fn new(layer_dir: &PathBuf) -> Self {
        Self {
            root: layer_dir.clone(),
            launch: Launch::new(),
        }
    }

    pub fn launch_path(&self) -> PathBuf {
        self.root.join(LAUNCH_TOML_FILE)
    }

    pub fn add(&mut self, name: &str) -> Result<Layer, std::io::Error> {
        let layer = Layer::new(self.root.to_str().unwrap(), name)?;

        Ok(layer)
    }

    pub fn write_launch(&self) -> Result<(), std::io::Error> {
        let string = toml::to_string(&self.launch).unwrap();
        let mut file = File::create(&self.launch_path())?;
        file.write_all(string.as_bytes())?;

        Ok(())
    }
}

pub fn new(name: &str) -> Result<Layer, std::io::Error> {
    Layer::new(ROOT_LAYER_FOLDER, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[test]
    fn it_adds_new_layer() {
        assert!(new("foo").is_ok());
        assert!(PathBuf::from("/layers/foo").is_dir());
    }

    #[test]
    fn it_writes_launch_toml() {
        let tmp_dir = TempDir::new("libbuildpack.rs").unwrap();
        let root = tmp_dir.path().join("layers").join("buildpack");
        std::fs::create_dir_all(&root).unwrap();
        let mut layers = Layers::new(&root);
        layers.launch.add_process("web", "bin/rails");

        assert!(layers.write_launch().is_ok());
        assert!(root.join("launch.toml").is_file());
    }
}
