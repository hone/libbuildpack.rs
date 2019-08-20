use std::fs;
use std::path::{Path, PathBuf};

use crate::build_plan::BuildPlan;
use crate::buildpack::Buildpack;
use crate::error::{Error, ErrorKind, Result};
use crate::layers::Layers;
use crate::platform::Platform;
use crate::stack::Stack;

const SUCCESS_STATUS_CODE: i32 = 0;

const BUILDPACK_FILE: &str = "buildpack.toml";

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
    ) -> Result<Self> {
        let buildpack = Buildpack::from_file(Self::find_toml()?)?;

        Ok(Self {
            root: std::env::current_dir()?,
            build_plan: BuildPlan::new(),
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

        Ok(SUCCESS_STATUS_CODE)
    }

    pub fn fail(&self, code: i32) -> i32 {
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
    use failure::Error;
    use tempdir::TempDir;

    use crate::build_plan::Dependency;

    use std::result::Result;

    #[test]
    fn build_succeeds_writes_a_build_plan() -> Result<(), Error> {
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

        std::env::set_var("CNB_STACK_ID", "aspen");
        let build = Build::new(&layers_dir, &platform_dir, build_plan)?;
        std::env::remove_var("CNB_STACK_ID");

        let mut build_plan = BuildPlan::new();
        build_plan.insert("ruby", Dependency::new("2.6.3"));

        let result = build.success(&build_plan);

        assert!(result.is_ok());

        let string = fs::read_to_string(build.build_plan_output)?;
        let written_build_plan: BuildPlan = toml::from_str(&string)?;
        assert_eq!(written_build_plan.get("ruby").unwrap().version, "2.6.3");

        fs::remove_file(&buildpack_toml_path)?;

        Ok(())
    }
}
