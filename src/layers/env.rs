use std::collections::HashMap;
use std::env::VarError;
use std::ffi::{OsStr, OsString};

pub struct Envs {
    pub build: EnvSet,
    pub launch: EnvSet,
    pub shared: EnvSet,
}

impl Envs {
    pub fn new() -> Self {
        Self {
            build: EnvSet::new(),
            launch: EnvSet::new(),
            shared: EnvSet::new(),
        }
    }
}

pub struct EnvSet {
    pub append: Env,
    pub r#override: Env,
    pub append_path: Env,
}

impl EnvSet {
    pub fn new() -> Self {
        Self {
            append: Env::new(),
            r#override: Env::new(),
            append_path: Env::new(),
        }
    }
}

pub struct Env {
    env: HashMap<OsString, OsString>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
        }
    }

    pub fn vars(&self) -> impl Iterator<Item = (&str, &str)> {
        self.env
            .iter()
            .map(|(key, value)| (key.to_str().unwrap_or(""), value.to_str().unwrap_or("")))
    }

    pub fn vars_os(&self) -> impl Iterator<Item = (&OsString, &OsString)> {
        self.env.iter()
    }

    pub fn var(&self, key: &OsStr) -> Result<String, VarError> {
        match &self.env.get(key) {
            Some(value) => value
                .as_os_str()
                // only works if it's unicode
                .to_str()
                .map(|str| str.to_string())
                .ok_or(VarError::NotUnicode(value.as_os_str().to_os_string())),
            None => Err(VarError::NotPresent),
        }
    }

    pub fn var_os(&self, key: &OsStr) -> Option<OsString> {
        self.env.get(key).map(|value| OsString::from(value))
    }

    pub fn set_var(&mut self, key: &str, value: &str) {
        &self.env.insert(OsString::from(key), OsString::from(value));
    }

    pub fn set_var_os(&mut self, key: &OsStr, value: &OsStr) {
        &self.env.insert(key.to_os_string(), value.to_os_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_var() {
        let mut internal_env = HashMap::new();
        internal_env.insert(OsString::from("FOO"), OsString::from("foo"));
        let env = Env { env: internal_env };

        let value = env.var(OsStr::new("FOO"));
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), String::from("foo"));
    }

    #[test]
    fn it_gets_var_none() {
        let env = Env::new();
        let value = env.var(OsStr::new("FOO"));
        assert!(value.is_err());
        assert_eq!(value, Err(VarError::NotPresent));
    }

    #[test]
    fn it_gets_var_os() {
        let mut internal_env = HashMap::new();
        internal_env.insert(OsString::from("FOO"), OsString::from("foo"));
        let env = Env { env: internal_env };

        let value = env.var_os(OsStr::new("FOO"));
        assert!(value.is_some());
        assert_eq!(value.unwrap(), OsString::from("foo"));
    }

    #[test]
    fn it_gets_var_os_none() {
        let env = Env::new();
        let value = env.var_os(OsStr::new("FOO"));
        assert!(value.is_none());
    }

    #[test]
    fn it_sets_var() {
        let mut env = Env::new();
        env.set_var("FOO", "BAR");

        assert_eq!(env.env.get(OsStr::new("FOO")).unwrap(), "BAR");

        env.set_var("FOO", "foo");
        assert_eq!(env.env.get(OsStr::new("FOO")).unwrap(), "foo");
    }

    #[test]
    fn it_sets_var_os() {
        let mut env = Env::new();
        env.set_var_os(OsStr::new("FOO"), OsStr::new("BAR"));

        assert_eq!(env.env.get(OsStr::new("FOO")).unwrap(), "BAR");

        env.set_var_os(OsStr::new("FOO"), OsStr::new("foo"));
        assert_eq!(env.env.get(OsStr::new("FOO")).unwrap(), "foo");
    }

    #[test]
    fn it_iterates_vars_os() {
        let mut internal_env = HashMap::new();
        internal_env.insert(OsString::from("FOO"), OsString::from("foo"));
        let env = Env { env: internal_env };

        for (key, value) in env.vars_os() {
            assert_eq!(key, &OsString::from("FOO"));
            assert_eq!(value, &OsString::from("foo"));
        }
    }

    #[test]
    fn it_iterates_vars() {
        let mut internal_env = HashMap::new();
        internal_env.insert(OsString::from("FOO"), OsString::from("foo"));
        let env = Env { env: internal_env };

        for (key, value) in env.vars() {
            assert_eq!(key, String::from("FOO"));
            assert_eq!(value, String::from("foo"));
        }
    }
}
