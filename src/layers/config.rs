use crate::metadata::Metadata;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub build: Option<bool>,
    pub cache: Option<bool>,
    pub launch: Option<bool>,
    metadata: Option<Metadata>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            launch: None,
            build: None,
            cache: None,
            metadata: None,
        }
    }

    pub fn metadata_as_mut(&mut self) -> &mut Metadata {
        if self.metadata.is_none() {
            self.metadata = Some(Metadata::new());
        }

        self.metadata.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_inserts_and_fetches_toml_value() {
        let mut metadata = Metadata::new();
        metadata.insert("foo", "bar");

        let mut config = Config::new();
        config.metadata_as_mut().insert("foo", "bar");

        assert_eq!(
            Some(&toml::Value::String("bar".to_string())),
            metadata.get("foo")
        );
    }
}
