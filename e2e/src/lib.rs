//! End-to-end tests of the vertigo-forms storybook.
//!
//! Each test binary (`tests/storybook`) is one suite. It builds the storybook from the current
//! workspace, serves it with vertigo-cli in this process (SSR included) and drives a local
//! Chrome through thirtyfour. How to run it: `README.md`.

pub mod browser;
pub mod build;
pub mod env;
pub mod resource_table;
pub mod runner;
pub mod storybook;

pub mod prelude {
    pub use std::{sync::Arc, time::Duration};

    pub use anyhow::{Context, Result, anyhow, bail, ensure};
    pub use serde_json::{Value, json};
    pub use thirtyfour::{By, Key, WebElement, components::SelectElement};

    pub use crate::{
        browser::{Ctx, normalize, xpath_literal},
        env::TestEnv,
        resource_table::{Table, click_in_form},
        runner::{TestCase, known_failures, run_suite},
    };
}
