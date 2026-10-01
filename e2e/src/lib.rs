//! End-to-end tests of the vertigo-forms storybook, on the harness of `vertigo-testing`.
//!
//! Each test binary (`tests/storybook`) is one suite. It builds the storybook from the current
//! workspace, serves it with vertigo-cli in this process (SSR included) and drives a local
//! Chrome through thirtyfour. How to run it: `README.md`.

// The helpers extend `Ctx` of the harness through traits. Their futures run on `block_on`, so
// they needn't be `Send`.
#![allow(async_fn_in_trait)]

pub mod env;
pub mod resource_table;
pub mod storybook;

pub use vertigo_testing::browser_tests;

/// What a test gets: a Chrome session and the helpers of `vertigo-testing`.
pub type Ctx = vertigo_testing::Ctx<env::TestEnv>;

pub mod prelude {
    pub use vertigo_testing::prelude::*;

    pub use crate::{
        Ctx,
        env::TestEnv,
        resource_table::{Table, TableExt, click_in_form},
        storybook::StorybookExt,
    };
}
