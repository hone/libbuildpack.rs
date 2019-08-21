use crate::env::Env;

#[derive(Debug)]
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

#[derive(Debug)]
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

    pub fn clear(&mut self) {
        self.append.clear();
        self.r#override.clear();
        self.append_path.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::VarError;

    #[test]
    fn it_clears_env_set() {
        let mut env_set = EnvSet::new();
        env_set.append.set_var("FOO", "foo");
        env_set.append_path.set_var("FOO", "foo");
        env_set.r#override.set_var("FOO", "foo");

        env_set.clear();

        assert_eq!(env_set.append.var("FOO"), Err(VarError::NotPresent));
        assert_eq!(env_set.append_path.var("FOO"), Err(VarError::NotPresent));
        assert_eq!(env_set.r#override.var("FOO"), Err(VarError::NotPresent));
    }
}
