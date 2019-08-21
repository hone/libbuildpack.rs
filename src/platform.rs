use crate::{env::Env, error::Result};
use log::debug;
use std::{fs, path::Path};

const ENV_DIR: &str = "env";

#[derive(Debug)]
pub struct Platform {
    pub env: Env,
}

impl Platform {
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let mut env = Env::new();
        let env_dir = dir.as_ref().join(ENV_DIR);

        if env_dir.exists() {
            for entry in env_dir.read_dir()? {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    // file_stem() returns None if no file name
                    if let Some(key) = path.file_stem() {
                        let value = fs::read_to_string(&path)?;
                        env.set_var(key, value);
                    }
                }
            }
        }

        debug!("Platform environment variables: {:#?}", env);

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
    use failure::Error;
    use std::{path::PathBuf, result::Result};
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
    fn it_doesnt_blow_up_on_no_env_dir() -> Result<(), Error> {
        let tmpdir = TempDir::new("platform")?;
        let platform_dir = tmpdir.path().join("");
        let platform = Platform::new(platform_dir);

        assert!(platform.is_ok());

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
