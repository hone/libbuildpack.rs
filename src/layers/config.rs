use crate::metadata::Metadata;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub build: bool,
    #[serde(default)]
    pub cache: bool,
    #[serde(default)]
    pub launch: bool,
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    #[serde(default)]
    pub metadata: Metadata,
}

impl Config {
    pub fn new() -> Self {
        Self {
            launch: false,
            build: false,
            cache: false,
            metadata: Metadata::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_metadata_to_toml() {
        let mut config = Config::new();
        config.metadata.insert("foo", "bar");

        let toml_string = toml::to_string(&config);
        assert!(toml_string.is_ok());
        if let Ok(string) = toml_string {
            assert_eq!(
                r#"build = false
cache = false
launch = false

[metadata]
foo = "bar"
"#,
                string
            );
        }
    }

    #[test]
    fn it_doesnt_serialize_metadata_to_toml() {
        let mut config = Config::new();
        config.launch = true;

        let toml_string = toml::to_string(&config);
        assert!(toml_string.is_ok());
        if let Ok(string) = toml_string {
            assert_eq!(
                r#"build = false
cache = false
launch = true
"#,
                string
            );
        }
    }
}
