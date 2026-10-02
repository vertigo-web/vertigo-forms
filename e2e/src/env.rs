//! What one suite runs against: the storybook, built from the current workspace and served
//! by vertigo-cli in this process, and a chromedriver shared by the tests.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use actix_web::{App, HttpServer, dev::ServerHandle, rt::System};
use anyhow::{Context, Result, anyhow};
use thirtyfour::{
    ChromiumLikeCapabilities, DesiredCapabilities, LoggingPrefsLogLevel, WebDriver,
    extensions::query::ElementPollerWithTimeout,
    manager::{BrowserKind, WebDriverManager},
};
use vertigo_cli::serve::{MountConfigBuilder, ServerState, vertigo_install};

use crate::build;

/// Time limit of element queries and of the `Ctx::wait_*` helpers.
pub const WAIT: Duration = Duration::from_secs(10);
pub const POLL: Duration = Duration::from_millis(100);

/// Settings from the `E2E_*` environment variables.
#[derive(Debug, Clone)]
pub struct Settings {
    pub target_dir: PathBuf,
    pub release: bool,
    pub skip_build: bool,
    pub headless: bool,
    pub chromedriver: Option<PathBuf>,
}

impl Settings {
    pub fn from_env() -> Self {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the e2e crate lives in the workspace");

        let target_dir = match std::env::var_os("CARGO_TARGET_DIR") {
            Some(dir) => workspace_root.join(dir),
            None => workspace_root.join("target"),
        };

        Self {
            target_dir,
            release: flag("E2E_RELEASE", false),
            skip_build: flag("E2E_SKIP_BUILD", false),
            headless: flag("E2E_HEADLESS", true),
            chromedriver: std::env::var_os("E2E_CHROMEDRIVER").map(PathBuf::from),
        }
    }
}

fn flag(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => !matches!(value.as_str(), "" | "0" | "false" | "no"),
        Err(_) => default,
    }
}

pub struct TestEnv {
    pub suite: &'static str,
    pub settings: Settings,
    /// `http://127.0.0.1:<port>`, without a trailing slash.
    pub base_url: String,
    /// `target/e2e/<suite>`: artifacts of failed tests.
    pub work_dir: PathBuf,
    webdriver: Arc<WebDriverManager>,
    server: ServerHandle,
}

impl TestEnv {
    pub async fn start(suite: &'static str) -> Result<Self> {
        let settings = Settings::from_env();

        let work_dir = settings.target_dir.join("e2e").join(suite);
        std::fs::create_dir_all(&work_dir)
            .with_context(|| format!("can't create {}", work_dir.display()))?;
        // Only the failures of this run belong there
        let _ = std::fs::remove_dir_all(work_dir.join("artifacts"));

        let build_dir = {
            let settings = settings.clone();
            tokio::task::spawn_blocking(move || build::build_storybook(&settings)).await??
        };

        let (base_url, server) = serve(&build_dir).await?;
        println!("Storybook served at {base_url}");

        Ok(Self {
            suite,
            webdriver: webdriver_manager(&settings),
            settings,
            base_url,
            work_dir,
            server,
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// A new Chrome session. Each test gets its own.
    pub async fn new_browser(&self) -> Result<WebDriver> {
        let mut caps = DesiredCapabilities::chrome();
        if self.settings.headless {
            caps.set_headless()?;
        }
        for arg in [
            "--window-size=1600,1000",
            "--lang=en-US",
            "--no-first-run",
            "--no-default-browser-check",
            "--disable-search-engine-choice-screen",
            // The tests don't depend on the network. The only outside resources are the
            // sample pictures from picsum.photos, so tests showing them allow the load error.
            "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1",
        ] {
            caps.add_arg(arg)?;
        }
        // After typing into the login forms the password manager may offer to save the
        // password, and its bubble intercepts clicks meant for the page
        caps.add_experimental_option(
            "prefs",
            serde_json::json!({
                "credentials_enable_service": false,
                "profile.password_manager_enabled": false,
                "profile.password_manager_leak_detection": false,
            }),
        )?;
        caps.add_arg("--disable-features=PasswordLeakDetection")?;
        // The console is checked after each test
        caps.set_browser_log_level(LoggingPrefsLogLevel::All)?;

        self.webdriver
            .launch(caps)
            .await
            .context("can't start Chrome through chromedriver")
    }

    /// Stops the server.
    pub async fn shutdown(&self) {
        self.server.stop(false).await;
    }
}

/// Serves `build_dir` as `vertigo serve` does, SSR included, but on a free port.
///
/// The directory is given by its absolute path, so the static files are served under it.
async fn serve(build_dir: &Path) -> Result<(String, ServerHandle)> {
    let mount_config = MountConfigBuilder::new("/", build_dir.to_string_lossy())
        .build()
        .map_err(|err| anyhow!("can't read the build in {}: {err:?}", build_dir.display()))?;

    // Compiles the wasm module for SSR, which takes a while for a debug build
    {
        let mount_config = mount_config.clone();
        tokio::task::spawn_blocking(move || ServerState::init(&mount_config))
            .await?
            .map_err(|err| anyhow!("can't load the storybook for SSR: {err:?}"))?;
    }

    let server =
        HttpServer::new(move || App::new().configure(|cfg| vertigo_install(cfg, &mount_config)))
            .workers(2)
            .disable_signals()
            .bind(("127.0.0.1", 0))
            .context("can't open a port for the storybook")?;
    let port = server
        .addrs()
        .first()
        .context("the server has no address")?
        .port();

    let server = server.run();
    let handle = server.handle();
    // Like in `vertigo serve`, actix gets a thread with its own system
    std::thread::spawn(move || System::new().block_on(server));

    Ok((format!("http://127.0.0.1:{port}"), handle))
}

fn webdriver_manager(settings: &Settings) -> Arc<WebDriverManager> {
    let mut builder = WebDriverManager::builder()
        .match_local()
        .poller(Arc::new(ElementPollerWithTimeout::new(WAIT, POLL)));
    if let Some(chromedriver) = &settings.chromedriver {
        builder = builder.driver_binary(BrowserKind::Chrome, chromedriver);
    }
    builder.build()
}
