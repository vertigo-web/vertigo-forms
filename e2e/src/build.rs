//! Builds the storybook as `vertigo build` does, with the vertigo-cli library of the same
//! version that serves it.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, ensure};
use vertigo_cli::{
    BuildOpts, CommonOpts,
    build::{self, BuildOptsInner},
};

use crate::env::Settings;

pub const STORYBOOK: &str = "vertigo-forms-storybook";

/// Builds the storybook into `target/e2e/<profile>/build` and returns that directory.
pub fn build_storybook(settings: &Settings) -> Result<PathBuf> {
    let profile = if settings.release { "release" } else { "debug" };
    let dest_dir = settings.target_dir.join("e2e").join(profile).join("build");

    if settings.skip_build {
        println!("E2E_SKIP_BUILD is set, using {}", dest_dir.display());
    } else {
        build_app(settings, STORYBOOK, &dest_dir)?;
    }

    let index = dest_dir.join("index.json");
    ensure!(index.is_file(), "missing {}", index.display());

    Ok(dest_dir)
}

fn build_app(settings: &Settings, package: &str, dest_dir: &Path) -> Result<()> {
    println!("Building {package} into {}", dest_dir.display());

    let opts = BuildOpts {
        inner: BuildOptsInner {
            package_name: Some(package.to_string()),
            public_path: None,
            // Optimization takes long and changes nothing the tests look at
            wasm_opt: Some(false),
            release_mode: Some(settings.release),
            wasm_run_source_map: false,
            cargo_opts: vec![],
        },
        common: CommonOpts {
            dest_dir: dest_dir.to_string_lossy().into_owned(),
            log_local_time: None,
        },
    };

    build::run(opts).map_err(|err| anyhow!("building {package} failed: {err:?}"))
}
