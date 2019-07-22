mod config;
mod env;
mod launch;
mod layer;
use crate::error::{Error, ErrorKind, Result};
use launch::Launch;
pub use layer::Layer;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const ROOT_LAYER_FOLDER: &str = "/layers";
const LAUNCH_TOML_FILE: &str = "launch.toml";

pub struct Layers {
    root: PathBuf,
    pub launch: Launch,
}

impl Layers {
    pub fn new<P: AsRef<Path>>(layer_dir: P) -> Self {
        Self {
            root: layer_dir.as_ref().to_path_buf(),
            launch: Launch::new(),
        }
    }

    pub fn launch_path(&self) -> PathBuf {
        self.root.join(LAUNCH_TOML_FILE)
    }

    pub fn add(&mut self, name: &str) -> Result<Layer> {
        if let Some(root) = self.root.to_str() {
            let layer = Layer::new(root, name)?;
            Ok(layer)
        } else {
            Err(Error::from(ErrorKind::new_path(&self.root)))
        }
    }

    pub fn write_launch(&self) -> Result<()> {
        let string = toml::to_string(&self.launch)?;
        let mut file = File::create(&self.launch_path())?;
        file.write_all(string.as_bytes())?;

        Ok(())
    }
}

pub fn new(name: &str) -> Result<Layer> {
    Layer::new(ROOT_LAYER_FOLDER, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;
    use std::result::Result;
    use tempdir::TempDir;

    #[test]
    fn it_adds_new_layer() {
        assert!(new("foo").is_ok());
        assert!(PathBuf::from("/layers/foo").is_dir());
    }

    #[test]
    fn it_writes_launch_toml() -> Result<(), Error> {
        let tmp_dir = TempDir::new("libbuildpack.rs")?;
        let root = tmp_dir.path().join("layers").join("buildpack");
        std::fs::create_dir_all(&root)?;
        let mut layers = Layers::new(&root);
        layers.launch.add_process("web", "bin/rails");

        assert!(layers.write_launch().is_ok());
        assert!(root.join("launch.toml").is_file());

        Ok(())
    }
}
