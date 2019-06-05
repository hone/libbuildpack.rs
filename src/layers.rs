mod config;
mod env;
mod layer;
pub use layer::Layer;

const ROOT_LAYER_FOLDER: &str = "/layers";

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
