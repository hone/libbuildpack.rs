use std::ops::{Deref, DerefMut};

use serde_derive::{Deserialize, Serialize};
use toml::map::Map;
use toml::value::Value;

#[derive(Serialize, Deserialize, Default, Debug)]
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

    pub fn insert<K: Into<String>, V: Into<Value>>(&mut self, key: K, value: V) {
        let key_string = key.into();
        let value_toml = value.into();
        self.0.insert(key_string, value_toml);
    }

    pub fn get<K: Into<String>>(&self, key: K) -> Option<&Value> {
        let key_string = key.into();
        self.0.get(&key_string)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_inserts_and_fetches_toml_value() {
        let mut metadata = Metadata::new();
        metadata.insert("foo", "bar");

        assert_eq!(
            Some(&toml::Value::String("bar".to_string())),
            metadata.get("foo")
        );
    }
}
