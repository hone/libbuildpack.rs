use crate::error::{ErrorKind, Result};
use crate::metadata::Metadata;

use std::fs;
use std::path::Path;

use log::debug;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Buildpack {
    #[serde(rename = "buildpack")]
    pub info: Info,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stacks: Vec<Stack>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
}

impl Buildpack {
    pub fn new<I: Into<String>, N: Into<String>, V: Into<String>>(
        id: I,
        name: N,
        version: V,
    ) -> Self {
        let buildpack = Self {
            info: Info::new(id, name, version),
            stacks: Vec::new(),
            metadata: Metadata::new(),
        };

        debug!("Buildpack: {:#?}", buildpack);

        buildpack
    }

    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let file_path = file.as_ref();
        let toml_string = fs::read_to_string(file_path)
            .map_err(|_| ErrorKind::FileNotFound(file_path.to_path_buf()))?;
        let buildpack: Buildpack = toml::from_str(&toml_string)?;

        debug!("Buildpack: {:#?}", buildpack);

        Ok(buildpack)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    pub id: String,
    pub name: String,
    pub version: String,
}

impl Info {
    pub fn new<I: Into<String>, N: Into<String>, V: Into<String>>(
        id: I,
        name: N,
        version: V,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stack {
    pub id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub mixins: Vec<String>,
    #[serde(rename = "build-images")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub build_images: Vec<String>,
    #[serde(rename = "run-images")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub run_images: Vec<String>,
}

impl Stack {
    pub fn new<I: Into<String>>(id: I) -> Self {
        Self {
            id: id.into(),
            mixins: Vec::new(),
            build_images: Vec::new(),
            run_images: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    use failure::Error;
    use std::result::Result;

    #[test]
    fn stack_serializes_to_toml() {
        let stack = Stack::new("aspen");

        let toml_string = toml::to_string(&stack);
        assert!(toml_string.is_ok());
        if let Ok(toml_string) = toml_string {
            assert_eq!(
                r#"id = "aspen"
"#,
                toml_string
            );
        }
    }

    #[test]
    fn stack_deserializes_from_toml() -> Result<(), Error> {
        let toml_string = r#"id = "aspen""#;
        let stack: Stack = toml::from_str(&toml_string)?;

        assert_eq!(stack.id, "aspen");

        Ok(())
    }

    #[test]
    fn buildpack_serializes_to_toml_bare() {
        let buildpack = Buildpack::new("heroku/ruby", "Heroku Ruby", "1.0.0");

        let toml_string = toml::to_string_pretty(&buildpack);
        assert!(toml_string.is_ok());
        if let Ok(toml_string) = toml_string {
            assert_eq!(
                r#"[buildpack]
id = 'heroku/ruby'
name = 'Heroku Ruby'
version = '1.0.0'
"#,
                toml_string
            );
        }
    }

    #[test]
    fn buildpack_serializes_to_toml() {
        let mut buildpack = Buildpack::new("heroku/ruby", "Heroku Ruby", "1.0.0");
        let mut heroku18 = Stack::new("heroku-18");
        heroku18
            .build_images
            .push("heroku/heroku-18:build".to_string());
        heroku18.run_images.push("heroku/heroku-18".to_string());
        buildpack.stacks.push(heroku18);
        let mut heroku16 = Stack::new("heroku-16");
        heroku16
            .build_images
            .push("heroku/heroku-16:build".to_string());
        heroku16.run_images.push("heroku/heroku-16".to_string());
        buildpack.stacks.push(heroku16);

        let toml_string = toml::to_string(&buildpack);
        assert!(toml_string.is_ok());
        if let Ok(toml_string) = toml_string {
            assert_eq!(
                r#"[buildpack]
id = "heroku/ruby"
name = "Heroku Ruby"
version = "1.0.0"

[[stacks]]
id = "heroku-18"
build-images = ["heroku/heroku-18:build"]
run-images = ["heroku/heroku-18"]

[[stacks]]
id = "heroku-16"
build-images = ["heroku/heroku-16:build"]
run-images = ["heroku/heroku-16"]
"#,
                toml_string
            );
        }
    }

    #[test]
    fn buildpack_deserialize_from_toml() -> Result<(), Error> {
        let toml_string = r#"[buildpack]
id = 'heroku/ruby'
name = 'Heroku Ruby'
version = '1.0.0'
"#;
        let buildpack: Buildpack = toml::from_str(&toml_string)?;

        assert_eq!("heroku/ruby", buildpack.info.id);

        Ok(())
    }

    #[test]
    fn buildpack_from_file_returns_not_found_error() -> Result<(), Error> {
        let temp_dir = TempDir::new("buildpack")?;

        let result = Buildpack::from_file(temp_dir.path());
        assert!(result.is_err());

        Ok(())
    }
}
