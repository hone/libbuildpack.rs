use std::ops::{Deref, DerefMut};

use serde_derive::{Deserialize, Serialize};
use toml::map::Map;
use toml::value::Value;

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

#[derive(Serialize, Deserialize)]
pub struct Metadata(Map<String, Value>);

impl Deref for Metadata {
    type Target = Map<String, Value>;

    fn deref(&self) -> &Map<String, Value> {
        &self.0
    }
}

impl DerefMut for Metadata {
    fn deref_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.0
    }
}

impl Metadata {
    pub fn new() -> Self {
        Metadata(Map::new())
    }

    pub fn insert<K: Into<String>, V: Into<toml::Value>>(&mut self, key: K, value: V) {
        let key_string = key.into();
        let value_toml = value.into();
        self.0.insert(key_string, value_toml);
    }

    pub fn get<K: Into<String>>(&self, key: K) -> Option<&toml::Value> {
        let key_string = key.into();
        self.0.get(&key_string)
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
