//! What one suite runs against: the storybook, built from the current workspace and served
//! by vertigo-cli in this process.

use std::ops::Deref;

use anyhow::Result;
use vertigo_testing::{ChromeConfig, Core, Env, Settings, build, serve::Server};

pub const STORYBOOK: &str = "vertigo-forms-storybook";

pub struct TestEnv {
    core: Core,
    server: Server,
}

impl Env for TestEnv {
    async fn start(suite: &'static str) -> Result<Self> {
        let settings = Settings::from_env(env!("CARGO_MANIFEST_DIR"))?;

        let build_dir = settings.build_dir().join("build");
        build::vertigo_app(&settings, STORYBOOK, &build_dir).await?;

        let server = Server::start(&build_dir).await?;
        println!("Storybook served at {}", server.base_url);

        let chrome = ChromeConfig {
            window_size: (1600, 1000),
            lang: "en-US",
            // The tests don't depend on the network. The only outside resources are the
            // sample pictures from picsum.photos, so tests showing them allow the load error.
            args: &["--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1"],
        };
        let core = Core::new(suite, settings, server.base_url.clone(), chrome)?;

        Ok(Self { core, server })
    }

    async fn shutdown(&self) {
        self.server.stop().await;
    }
}

impl Deref for TestEnv {
    type Target = Core;

    fn deref(&self) -> &Core {
        &self.core
    }
}
