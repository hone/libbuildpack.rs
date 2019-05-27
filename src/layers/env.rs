use std::collections::HashMap;

pub struct Envs {
    pub build: Env,
    pub launch: Env,
    pub shared: Env,
}

impl Envs {
    pub fn new() -> Self {
        Self {
            build: Env::new(),
            launch: Env::new(),
            shared: Env::new(),
        }
    }
}

pub struct Env {
    pub append: HashMap<String, String>,
    pub r#override: HashMap<String, String>,
    pub append_path: HashMap<String, String>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            append: HashMap::new(),
            r#override: HashMap::new(),
            append_path: HashMap::new(),
        }
    }
}
