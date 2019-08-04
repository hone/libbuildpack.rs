use crate::metadata::Metadata;

use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_derive::{Deserialize as DeriveDeserialize, Serialize as DeriveSerialize};

pub struct BuildPlan(HashMap<String, Dependency>);

impl BuildPlan {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn insert<S: Into<String>>(&mut self, key: S, value: Dependency) {
        self.0.insert(key.into(), value);
    }
}

impl Deref for BuildPlan {
    type Target = HashMap<String, Dependency>;

    fn deref(&self) -> &HashMap<String, Dependency> {
        &self.0
    }
}

impl DerefMut for BuildPlan {
    fn deref_mut(&mut self) -> &mut HashMap<String, Dependency> {
        &mut self.0
    }
}

impl Serialize for BuildPlan {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(&k, &v)?;
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for BuildPlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BuildPlanVisitor;

        impl<'de> Visitor<'de> for BuildPlanVisitor {
            type Value = BuildPlan;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("not a valid build plan")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut build_plan = BuildPlan::new();

                while let Some((key, value)) = map.next_entry::<String, toml::Value>()? {
                    let value: toml::Value = value;
                    let table = value.try_into::<toml::value::Table>().map_err(|_| {
                        serde::de::Error::invalid_type(
                            serde::de::Unexpected::NewtypeVariant,
                            &"Not a TOML Table",
                        )
                    })?;
                    let version = table
                        .get("version")
                        .ok_or_else(|| serde::de::Error::missing_field("version"))?
                        .as_str()
                        .ok_or_else(|| {
                            serde::de::Error::invalid_type(
                                serde::de::Unexpected::Other("Unknown"),
                                &"String",
                            )
                        })?;
                    let mut dependency = Dependency::new(version);

                    if let Some(metadata) = table.get("metadata") {
                        let metadata_table = metadata.as_table().ok_or_else(|| {
                            serde::de::Error::invalid_type(
                                serde::de::Unexpected::Other("Unknown"),
                                &"toml::value::Table",
                            )
                        })?;

                        for (key, value) in metadata_table {
                            dependency.metadata_as_mut().insert(key, value.clone());
                        }
                    }
                    build_plan.insert(key, dependency);
                }

                Ok(build_plan)
            }
        }

        deserializer.deserialize_map(BuildPlanVisitor)
    }
}

#[derive(DeriveSerialize, DeriveDeserialize)]
pub struct Dependency {
    pub version: String,
    metadata: Option<Metadata>,
}

impl Dependency {
    pub fn new<S: Into<String>>(version: S) -> Self {
        Self {
            version: version.into(),
            metadata: None,
        }
    }

    pub fn metadata_as_mut(&mut self) -> &mut Metadata {
        if self.metadata.is_none() {
            self.metadata = Some(Metadata::new());
        }

        self.metadata.as_mut().unwrap()
    }

    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use failure::Error;

    #[test]
    fn it_serializes_into_toml() -> Result<(), Error> {
        let mut build_plan = BuildPlan::new();
        build_plan.insert("alpha", Dependency::new("alpha"));

        let toml_string = toml::to_string(&build_plan);
        assert!(toml_string.is_ok());

        if let Ok(string) = toml_string {
            assert_eq!(
                string,
                r#"[alpha]
version = "alpha"
"#
            );
        }

        Ok(())
    }

    #[test]
    fn it_parses_toml_for_dependency() -> Result<(), Error> {
        let toml_string = r#"
version = "alpha"
"#;
        let dependency_result: Result<Dependency, toml::de::Error> = toml::from_str(&toml_string);

        assert!(dependency_result.is_ok());
        if let Ok(dependency) = dependency_result {
            assert_eq!(dependency.version, "alpha");
        }

        Ok(())
    }

    #[test]
    fn it_parses_toml_with_no_metadata() -> Result<(), Error> {
        let toml_string = r#"
[alpha]
version = "alpha"
    "#;

        let build_plan_result: Result<BuildPlan, toml::de::Error> = toml::from_str(&toml_string);

        assert!(build_plan_result.is_ok());
        if let Ok(build_plan) = build_plan_result {
            assert_eq!(build_plan.len(), 1);
            assert_eq!(build_plan.get("alpha").unwrap().version, "alpha");
        }

        Ok(())
    }

    #[test]
    fn it_parses_toml_with_metadata() -> Result<(), Error> {
        let toml_string = r#"
[alpha]
version = "alpha"

[alpha.metadata]
foo = "bar"
"#;

        let build_plan_result: Result<BuildPlan, toml::de::Error> = toml::from_str(&toml_string);

        assert!(build_plan_result.is_ok());
        if let Ok(build_plan) = build_plan_result {
            let dependency = build_plan.get("alpha").unwrap();
            assert_eq!(dependency.version, "alpha");
            assert_eq!(
                dependency
                    .metadata()
                    .unwrap()
                    .get("foo")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                "bar"
            );
        }

        Ok(())
    }
}
