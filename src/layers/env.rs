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

    pub fn var<K: AsRef<OsStr>>(&self, key: K) -> Result<String, VarError> {
        let key_ref = key.as_ref();

        match &self.env.get(key_ref) {
            Some(value) => value
                .as_os_str()
                // only works if it's unicode
                .to_str()
                .map(|str| str.to_string())
                .ok_or(VarError::NotUnicode(value.as_os_str().to_os_string())),
            None => Err(VarError::NotPresent),
        }
    }

    pub fn var_os<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString> {
        let key_as_ref = key.as_ref();

        self.env.get(key_as_ref).map(|value| OsString::from(value))
    }

    pub fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(&mut self, key: K, value: V) {
        let key_os_string = key.as_ref().to_os_string();
        let value_os_string = value.as_ref().to_os_string();

        &self.env.insert(key_os_string, value_os_string);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_var() {
        let mut internal_env = HashMap::new();
        internal_env.insert(OsString::from("FOO"), OsString::from("foo"));
        let mut env = Env { env: internal_env };

        let value = env.var("FOO");
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), String::from("foo"));

        env.set_var("BAR", "bar");

        let bar = env.var("BAR");
        assert!(bar.is_ok());
        assert_eq!(&bar.unwrap(), "bar");
    }

    #[test]
    fn it_gets_var_none() {
        let env = Env::new();
        let value = env.var("FOO");
        assert!(value.is_err());
        assert_eq!(value, Err(VarError::NotPresent));
    }

    #[test]
    fn it_gets_var_os() {
        let mut internal_env = HashMap::new();
        internal_env.insert(OsString::from("FOO"), OsString::from("foo"));
        let mut env = Env { env: internal_env };

        let foo = env.var_os("FOO");
        assert!(foo.is_some());
        assert_eq!(foo.unwrap(), OsString::from("foo"));

        env.set_var("BAR", "bar");

        let bar = env.var_os("BAR");
        assert!(bar.is_some());
        assert_eq!(bar.unwrap(), OsString::from("bar"));
    }

    #[test]
    fn it_gets_var_os_none() {
        let env = Env::new();
        let value = env.var_os("FOO");
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
