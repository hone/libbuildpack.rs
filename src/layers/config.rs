use serde_derive::{Deserialize, Serialize};
use toml::map::Map;
use toml::value::Value;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub build: Option<bool>,
    pub cache: Option<bool>,
    pub launch: Option<bool>,
    metadata: Option<Map<String, Value>>,
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

    pub fn metadata_as_mut(&mut self) -> &mut Map<String, Value> {
        if self.metadata.is_none() {
            self.metadata = Some(Map::new());
        }

        self.metadata.as_mut().unwrap()
    }
}
