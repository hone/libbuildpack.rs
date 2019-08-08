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

    pub fn set_env(&self) {
        for (key, val) in self.env.vars_os() {
            std::env::set_var(key, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::result::Result;

    use failure::Error;
    use tempdir::TempDir;

    struct Setup {
        _temp_dir: TempDir,
        pub env_dir: PathBuf,
        pub platform_dir: PathBuf,
    }

    fn setup() -> Result<Setup, Error> {
        let tmp_dir = TempDir::new("platform")?;
        let platform_dir = tmp_dir.path().join("");
        let env_dir = platform_dir.join("env");
        fs::create_dir_all(&env_dir)?;

        Ok(Setup {
            _temp_dir: tmp_dir,
            env_dir: env_dir,
            platform_dir: platform_dir,
        })
    }

    #[test]
    fn it_reads_env() -> Result<(), Error> {
        let setup = setup()?;

        fs::write(setup.env_dir.join("FOO"), "BAR")?;
        fs::write(setup.env_dir.join("BAR"), "BAZ")?;

        let platform = Platform::new(setup.platform_dir);

        assert!(platform.is_ok());
        if let Ok(platform) = platform {
            assert_eq!(platform.env.var("FOO").unwrap(), "BAR");
            assert_eq!(platform.env.var("BAR").unwrap(), "BAZ");
        }

        Ok(())
    }

    #[test]
    fn it_sets_env() -> Result<(), Error> {
        let setup = setup()?;
        fs::write(setup.env_dir.join("FOO"), "BAR")?;
        fs::write(setup.env_dir.join("BAR"), "BAZ")?;

        let platform = Platform::new(setup.platform_dir)?;
        let old_vars = std::env::vars();

        platform.set_env();
        assert_eq!(std::env::var("FOO")?, "BAR");
        assert_eq!(std::env::var("BAR")?, "BAZ");

        for (key, _) in platform.env.vars_os() {
            std::env::remove_var(key);
        }
        for (key, value) in old_vars {
            std::env::set_var(key, value);
        }

        Ok(())
    }
}
