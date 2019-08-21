use crate::build_plan::BuildPlan;
use crate::error::Result;
use crate::platform::Platform;
use crate::stack::Stack;

use std::fs;
use std::path::PathBuf;

use log::debug;

const FAIL_STATUS_CODE: i32 = 100;
const PASS_STATUS_CODE: i32 = 0;

#[derive(Debug)]
pub struct Detect {
    pub stack: Stack,
    pub platform: Platform,
    pub build_plan: BuildPlan,
    pub build_plan_output: PathBuf,
}

impl Detect {
    pub fn new<P: Into<PathBuf>, L: Into<PathBuf>>(
        platform_dir: P,
        build_plan_output: L,
    ) -> Result<Self> {
        Ok(Self {
            stack: Stack::new()?,
            build_plan: BuildPlan::new(),
            platform: Platform::new(platform_dir.into())?,
            build_plan_output: build_plan_output.into(),
        })
    }

    pub fn fail(&self) -> i32 {
        debug!("Detection failed. Exiting with {}", FAIL_STATUS_CODE);
        FAIL_STATUS_CODE
    }

    pub fn pass(&self, build_plan: Option<&BuildPlan>) -> Result<i32> {
        if let Some(build_plan) = build_plan {
            let toml_string = toml::to_string(build_plan)?;
            fs::write(&self.build_plan_output, toml_string)?;
        }

        debug!("Detection passed. Exiting with {}", PASS_STATUS_CODE);

        Ok(PASS_STATUS_CODE)
    }

    pub fn error(&self, code: i32) -> i32 {
        debug!("Detection produced an error. Exiting with {}", code);
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::result::Result;

    use crate::build_plan::Dependency;

    use failure::Error;
    use tempdir::TempDir;

    struct Setup {
        pub platform_dir: PathBuf,
        pub build_plan: PathBuf,
        pub old_env_var: Option<std::ffi::OsString>,
        _tmpdir: TempDir,
    }

    fn setup() -> Result<Setup, Error> {
        let old_env_var = std::env::var_os("CNB_STACK_ID");
        std::env::set_var("CNB_STACK_ID", "aspen");
        let tmpdir = TempDir::new("detect")?;
        let platform_dir = tmpdir.path().join("platform");
        let build_plan = tmpdir.path().join("build_plan");
        fs::create_dir_all(&platform_dir)?;

        Ok(Setup {
            platform_dir: platform_dir,
            build_plan: build_plan,
            old_env_var: old_env_var,
            _tmpdir: tmpdir,
        })
    }

    fn reset_cnb_stack_id(old_env_var: Option<std::ffi::OsString>) {
        if let Some(env_var) = old_env_var {
            std::env::set_var("CNB_STACK_ID", env_var);
        } else {
            std::env::remove_var("CNB_STACK_ID")
        }
    }

    #[test]
    fn it_fails_if_no_stack() -> Result<(), Error> {
        let tmpdir = TempDir::new("detect")?;
        let platform_dir = tmpdir.path().join("platform");
        let build_plan = tmpdir.path().join("build_plan");
        fs::create_dir_all(&platform_dir)?;

        let detect = Detect::new(platform_dir, build_plan);

        println!("{:?}", std::env::var_os("CNB_STACK_ID"));
        assert!(detect.is_err());

        Ok(())
    }

    #[test]
    fn it_writes_build_plan_on_pass() -> Result<(), Error> {
        let setup = setup()?;
        let detect = Detect::new(setup.platform_dir, setup.build_plan)?;
        reset_cnb_stack_id(setup.old_env_var);

        let mut build_plan = BuildPlan::new();
        build_plan.insert("ruby", Dependency::new("2.6.3"));

        let result = detect.pass(Some(&build_plan));

        assert!(result.is_ok());
        if let Ok(result) = result {
            assert_eq!(0, result);
        }

        let toml_string = fs::read_to_string(detect.build_plan_output)?;
        let output_build_plan: BuildPlan = toml::from_str(&toml_string)?;
        assert_eq!(output_build_plan.get("ruby").unwrap().version, "2.6.3");

        Ok(())
    }
}
