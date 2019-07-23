use crate::env::Env;
use crate::error::Result;

use std::fs;
use std::path::Path;

const ENV_DIR: &str = "env";

pub struct Platform {
    pub env: Env,
}

impl Platform {
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let mut env = Env::new();

        for entry in dir.as_ref().join(ENV_DIR).read_dir()? {
            if let Ok(entry) = entry {
                let path = entry.path();

                // file_stem() returns None if no file name
                if let Some(key) = path.file_stem() {
                    let value = fs::read_to_string(&path)?;
                    env.set_var(key, value);
                }
            }
        }

        Ok(Self { env: env })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::result::Result;

    use failure::Error;
    use tempdir::TempDir;

    #[test]
    fn it_reads_env() -> Result<(), Error> {
        let platform_dir = TempDir::new("platform")?;
        let env_dir = platform_dir.path().join("env");
        fs::create_dir_all(&env_dir)?;

        fs::write(env_dir.join("FOO"), "BAR")?;
        fs::write(env_dir.join("BAR"), "BAZ")?;

        let platform = Platform::new(platform_dir);

        assert!(platform.is_ok());
        if let Ok(platform) = platform {
            assert_eq!(platform.env.var("FOO").unwrap(), "BAR");
            assert_eq!(platform.env.var("BAR").unwrap(), "BAZ");
        }

        Ok(())
    }
}
