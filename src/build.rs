use crate::{
    build_plan::BuildPlan,
    buildpack::Buildpack,
    error::{Error, ErrorKind, Result},
    layers::Layers,
    platform::Platform,
    stack::Stack,
};
use log::debug;
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

const SUCCESS_STATUS_CODE: i32 = 0;

const BUILDPACK_FILE: &str = "buildpack.toml";

#[derive(Debug)]
pub struct Build {
    pub root: PathBuf,
    pub build_plan: BuildPlan,
    pub build_plan_output: PathBuf,
    pub buildpack: Buildpack,
    pub layers: Layers,
    pub platform: Platform,
    pub stack: Stack,
}

impl Build {
    pub fn new<L: AsRef<Path>, P: AsRef<Path>, A: Into<PathBuf>>(
        layers: L,
        platform: P,
        plan: A,
        // need to use Box, so it can be `Sized` at compile time
        plan_reader: Option<Box<dyn Read>>,
    ) -> Result<Self> {
        let buildpack = Buildpack::from_file(Self::find_toml()?)?;
        let mut stdin_buf = String::new();
        let mut reader = plan_reader.unwrap_or(Box::new(io::stdin()));
        reader.read_to_string(&mut stdin_buf)?;
        let build_plan: BuildPlan = toml::from_str(&stdin_buf)?;

        Ok(Self {
            root: std::env::current_dir()?,
            build_plan: build_plan,
            build_plan_output: plan.into(),
            buildpack: buildpack,
            layers: Layers::new(layers.as_ref()),
            platform: Platform::new(platform.as_ref())?,
            stack: Stack::new()?,
        })
    }

    pub fn success(&self, build_plan: &BuildPlan) -> Result<i32> {
        let toml_string = toml::to_string(build_plan)?;

        fs::write(&self.build_plan_output, &toml_string)?;

        debug!("Build success. Exiting with {}", SUCCESS_STATUS_CODE);

        Ok(SUCCESS_STATUS_CODE)
    }

    pub fn fail(&self, code: i32) -> i32 {
        debug!("Build failed. Exiting with {}", code);

        code
    }

    fn find_toml() -> Result<PathBuf> {
        Ok(Self::buildpack_dir()?.join(BUILDPACK_FILE))
    }

    fn buildpack_dir() -> Result<PathBuf> {
        if let Some(bin_path) = std::env::args_os().nth(0) {
            let mut buildpack_dir = PathBuf::from(bin_path.into_string()?);
            buildpack_dir.pop();
            buildpack_dir.pop();
            Ok(buildpack_dir)
        } else {
            Err(Error::from(ErrorKind::NoArgs))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_plan::Dependency;
    use failure::Error;
    use std::result::Result;
    use tempdir::TempDir;

    struct Setup {
        pub buildpack_toml_path: PathBuf,
        pub layers_dir: PathBuf,
        pub platform_dir: PathBuf,
        pub build_plan_path: PathBuf,
        pub old_env_var: Option<std::ffi::OsString>,
        _tmpdir: TempDir,
    }

    fn setup() -> Result<Setup, Error> {
        let buildpack_toml_path = Build::find_toml()?;
        fs::write(
            &buildpack_toml_path,
            r#"[buildpack]
id = "heroku/ruby"
name = "Heroku Ruby"
version = "1.0.0"

[[stacks]]
id = "heroku-18"
build-images = ["heroku/heroku-18:build"]
run-images = ["heroku/heroku-18"]
"#,
        )?;

        let temp_dir = TempDir::new("build")?;
        let layers_dir = temp_dir.path().join("layers");
        let platform_dir = temp_dir.path().join("platform");
        let build_plan = temp_dir.path().join("build_plan");

        fs::create_dir_all(&layers_dir)?;
        fs::create_dir_all(&platform_dir)?;

        let old_env_var = std::env::var_os("CNB_STACK_ID");
        std::env::set_var("CNB_STACK_ID", "aspen");

        Ok(Setup {
            buildpack_toml_path: buildpack_toml_path,
            layers_dir: layers_dir,
            platform_dir: platform_dir,
            build_plan_path: build_plan,
            old_env_var: old_env_var,
            _tmpdir: temp_dir,
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
    fn build_succeeds_writes_a_build_plan() -> Result<(), Error> {
        let setup = setup()?;
        let stdin = b"";
        let build = Build::new(
            &setup.layers_dir,
            &setup.platform_dir,
            &setup.build_plan_path,
            Some(Box::new(&stdin[..])),
        )?;
        reset_cnb_stack_id(setup.old_env_var);

        let mut build_plan = BuildPlan::new();
        build_plan.insert("ruby", Dependency::new("2.6.3"));

        let result = build.success(&build_plan);

        assert!(result.is_ok());

        let string = fs::read_to_string(build.build_plan_output)?;
        let written_build_plan: BuildPlan = toml::from_str(&string)?;
        assert_eq!(written_build_plan.get("ruby").unwrap().version, "2.6.3");

        fs::remove_file(&setup.buildpack_toml_path)?;

        Ok(())
    }

    #[test]
    fn it_parses_build_plan() -> Result<(), Error> {
        let setup = setup()?;
        let stdin = r#"
[ruby]
version = "2.6.5"
    "#
        .as_bytes();

        let build = Build::new(
            &setup.layers_dir,
            &setup.platform_dir,
            &setup.build_plan_path,
            Some(Box::new(&stdin[..])),
        );

        reset_cnb_stack_id(setup.old_env_var);

        assert!(build.is_ok());
        if let Ok(build) = build {
            assert_eq!(build.build_plan.get("ruby").unwrap().version, "2.6.5",);
        }

        Ok(())
    }
}
