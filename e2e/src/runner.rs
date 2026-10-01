//! A libtest-compatible runner (filters, `--list`, `--test-threads`) which starts the
//! environment once per suite, lazily, and gives each test its own Chrome session.

use std::{future::Future, pin::Pin, sync::Arc};

use anyhow::{Result, anyhow};
use libtest_mimic::{Arguments, Failed, Trial};
use thirtyfour::testing::{BrowserTestError, run_browser_test};
use tokio::{runtime::Runtime, sync::OnceCell};

use crate::{browser::Ctx, env::TestEnv};

/// Not `Send`: each test is run by `block_on` on its own runner thread.
pub type TestFuture = Pin<Box<dyn Future<Output = Result<()>>>>;

pub struct TestCase {
    pub name: String,
    pub body: fn(Ctx) -> TestFuture,
    /// Skipped unless run with `--ignored` / `--include-ignored`.
    pub ignored: bool,
}

/// Tests of known bugs, not fixed yet: they describe the correct behavior, but are skipped by
/// default to keep the suite green.
pub fn known_failures(tests: Vec<TestCase>) -> Vec<TestCase> {
    tests
        .into_iter()
        .map(|test| TestCase {
            ignored: true,
            ..test
        })
        .collect()
}

/// `browser_tests![module::test, ...]`: tests taking a [`Ctx`].
#[macro_export]
macro_rules! browser_tests {
    ($($test:path),* $(,)?) => {
        vec![$($crate::runner::TestCase {
            name: stringify!($test).replace(' ', ""),
            body: |ctx| Box::pin($test(ctx)),
            ignored: false,
        }),*]
    };
}

type SharedEnv = Arc<OnceCell<Result<Arc<TestEnv>, String>>>;

pub fn run_suite(suite: &'static str, tests: Vec<TestCase>) -> ! {
    let mut args = Arguments::from_args();
    if args.test_threads.is_none() {
        let cpus = std::thread::available_parallelism().map_or(1, |n| n.get());
        args.test_threads = Some(cpus.min(4));
    }

    let runtime = Arc::new(Runtime::new().expect("can't start the tokio runtime"));
    let env: SharedEnv = Arc::default();

    let trials = tests
        .into_iter()
        .map(|test| {
            let runtime = runtime.clone();
            let env = env.clone();
            let name = test.name.clone();
            let ignored = test.ignored;
            Trial::test(name, move || {
                runtime
                    .block_on(run_test(suite, &env, test))
                    .map_err(|err| Failed::from(format!("{err:?}")))
            })
            .with_ignored_flag(ignored)
        })
        .collect();

    let conclusion = libtest_mimic::run(&args, trials);

    if let Some(Ok(env)) = env.get() {
        runtime.block_on(env.shutdown());
    }

    conclusion.exit()
}

async fn run_test(suite: &'static str, env: &SharedEnv, test: TestCase) -> Result<()> {
    let env = env
        .get_or_init(|| async {
            TestEnv::start(suite)
                .await
                .map(Arc::new)
                .map_err(|err| format!("{err:?}"))
        })
        .await
        .clone()
        .map_err(|err| anyhow!("the test environment didn't start:\n{err}"))?;

    let driver = env.new_browser().await?;
    let result = run_browser_test(async { Ok(driver) }, |driver| async move {
        let ctx = Ctx::new(env.clone(), driver, &test.name);
        let result = (test.body)(ctx.clone()).await;
        ctx.finish(result).await
    })
    .await;

    match result {
        Ok(()) => Ok(()),
        Err(BrowserTestError::Body(err)) => Err(err),
        Err(BrowserTestError::BodyAndCleanup { body, .. }) => Err(body),
        Err(err) => Err(anyhow!("{err}")),
    }
}
