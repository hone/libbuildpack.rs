mod config;
mod env;
mod layer;
pub use layer::Layer;

use std::collections::HashMap;
use std::path::PathBuf;

const ROOT_LAYER_FOLDER: &str = "/layers";

pub struct Layers {
    root: PathBuf,
    layers: HashMap<String, Layer>,
}

impl Layers {
    pub fn new(layer_dir: &PathBuf) -> Self {
        Self {
            root: layer_dir.clone(),
            layers: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str) -> Result<&mut Layer, std::io::Error> {
        let layer = Layer::new(self.root.to_str().unwrap(), name)?;
        &self.layers.insert(name.to_string(), layer);
        Ok(self.layers.get_mut(name).unwrap())
    }
}

pub fn new(name: &str) -> Result<Layer, std::io::Error> {
    Layer::new(ROOT_LAYER_FOLDER, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn it_adds_new_layer() {
        assert!(new("foo").is_ok());
        assert!(PathBuf::from("/layers/foo").is_dir());
    }
}
