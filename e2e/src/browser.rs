//! What a browser test gets: a Chrome session and helpers that know how a vertigo app
//! behaves.

use std::{
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use serde_json::Value;
use thirtyfour::{BrowserLogEntry, By, Key, WebDriver, WebElement, prelude::ElementQueryable};

use crate::env::{POLL, TestEnv, WAIT};

#[derive(Clone)]
pub struct Ctx {
    pub env: Arc<TestEnv>,
    pub driver: WebDriver,
    pub name: String,
    allowed_console: Arc<Mutex<Vec<String>>>,
}

impl Deref for Ctx {
    type Target = WebDriver;

    fn deref(&self) -> &WebDriver {
        &self.driver
    }
}

impl Ctx {
    pub(crate) fn new(env: Arc<TestEnv>, driver: WebDriver, name: &str) -> Self {
        Self {
            env,
            driver,
            name: name.to_string(),
            allowed_console: Default::default(),
        }
    }

    pub fn url(&self, path: &str) -> String {
        self.env.url(path)
    }

    /// Loads `path` and waits until the app takes over the page rendered by the server.
    pub async fn open(&self, path: &str) -> Result<()> {
        self.driver.goto(self.url(path)).await?;
        self.wait_booted().await
    }

    /// Reloads the page and waits until a new document replaces the old one and its app
    /// starts.
    pub async fn reload(&self) -> Result<()> {
        self.mark_document().await?;
        self.driver.refresh().await?;
        self.wait_for_new_document().await
    }

    /// Waits until the document marked by `mark_document` is replaced by another one, and
    /// until its app starts.
    pub async fn wait_for_new_document(&self) -> Result<()> {
        self.wait_for("a new document", async || {
            let old = self.js("return window.__e2eDocument === true").await?;
            Ok((old == Value::Bool(false)).then_some(()))
        })
        .await?;
        self.wait_booted().await
    }

    /// Until the WASM app mounts, the page rendered by the server is dead: nothing reacts to
    /// clicks or typing. Hydration publishes its report when the first DOM batch of the app
    /// arrives.
    pub async fn wait_booted(&self) -> Result<()> {
        self.wait_for(
            "the app to start (window.__vertigo_hydration)",
            async || {
                let booted = self.js("return window.__vertigo_hydration != null").await?;
                Ok((booted == Value::Bool(true)).then_some(()))
            },
        )
        .await
    }

    /// Polls `check` until it returns a value. Errors count as "not yet" (an element replaced
    /// by vertigo in the meantime is stale) until the time limit passes.
    pub async fn wait_for<T>(
        &self,
        what: &str,
        mut check: impl AsyncFnMut() -> Result<Option<T>>,
    ) -> Result<T> {
        let deadline = Instant::now() + WAIT;
        loop {
            let result = check().await;
            let timed_out = Instant::now() > deadline;
            match result {
                Ok(Some(value)) => return Ok(value),
                Ok(None) if timed_out => bail!("timed out waiting for {what}"),
                Err(err) if timed_out => {
                    return Err(err.context(format!("timed out waiting for {what}")));
                }
                _ => tokio::time::sleep(POLL).await,
            }
        }
    }

    pub async fn js(&self, script: &str) -> Result<Value> {
        self.js_with(script, vec![]).await
    }

    pub async fn js_with(&self, script: &str, args: Vec<Value>) -> Result<Value> {
        let ret = self
            .driver
            .execute(script, args)
            .await
            .with_context(|| format!("script failed: {script}"))?;
        Ok(ret.json().clone())
    }

    /// Visible text of the page, with whitespace normalized.
    pub async fn page_text(&self) -> Result<String> {
        let text = self.js("return document.body.innerText").await?;
        Ok(normalize(text.as_str().unwrap_or_default()))
    }

    pub async fn wait_for_text(&self, text: &str) -> Result<()> {
        let text = normalize(text);
        self.wait_for(&format!("text {text:?}"), async || {
            Ok(self.page_text().await?.contains(&text).then_some(()))
        })
        .await
    }

    pub async fn wait_for_no_text(&self, text: &str) -> Result<()> {
        let text = normalize(text);
        self.wait_for(&format!("text {text:?} to disappear"), async || {
            Ok((!self.page_text().await?.contains(&text)).then_some(()))
        })
        .await
    }

    /// First match; waits for it to appear.
    pub async fn find(&self, by: By) -> Result<WebElement> {
        Ok(self.driver.query(by).first().await?)
    }

    /// All matches at the moment, without waiting.
    pub async fn find_all(&self, by: By) -> Result<Vec<WebElement>> {
        Ok(self.driver.find_all(by).await?)
    }

    pub async fn wait_for_none(&self, by: By) -> Result<()> {
        self.wait_for(&format!("no {by}"), async || {
            Ok(self.find_all(by.clone()).await?.is_empty().then_some(()))
        })
        .await
    }

    /// First `tag` whose whole text is `text`. The client splits adjacent texts into separate
    /// nodes and SSR glues them together, so this compares the element's normalized text,
    /// not its text nodes.
    pub async fn by_text(&self, tag: &str, text: &str) -> Result<WebElement> {
        self.find(By::XPath(format!(
            "//{tag}[normalize-space(.)={}]",
            xpath_literal(text)
        )))
        .await
    }

    /// Normalized text of the first match.
    pub async fn text_of(&self, by: By) -> Result<String> {
        Ok(normalize(&self.find(by).await?.text().await?))
    }

    /// Waits until the first match has exactly the `expected` text.
    pub async fn wait_for_text_of(&self, by: By, expected: &str) -> Result<()> {
        let expected = normalize(expected);
        let mut last = None;
        self.wait_for(&format!("{by} with text {expected:?}"), async || {
            let text = self.text_of(by.clone()).await?;
            let matches = text == expected;
            last = Some(text);
            Ok(matches.then_some(()))
        })
        .await
        .with_context(|| format!("last text: {last:?}"))
    }

    /// Texts of all current matches.
    pub async fn texts(&self, by: By) -> Result<Vec<String>> {
        let mut texts = Vec::new();
        for element in self.find_all(by).await? {
            texts.push(normalize(&element.text().await?));
        }
        Ok(texts)
    }

    /// For each current match of `xpath`, the texts of its child elements. Read by one script,
    /// so vertigo can't replace a node halfway through.
    pub async fn children_texts(&self, xpath: &str) -> Result<Vec<Vec<String>>> {
        let value = self
            .js_with(
                "const found = document.evaluate(arguments[0], document, null,
                     XPathResult.ORDERED_NODE_SNAPSHOT_TYPE, null);
                 const texts = [];
                 for (let i = 0; i < found.snapshotLength; i++) {
                     texts.push(Array.from(found.snapshotItem(i).children, child => child.innerText));
                 }
                 return texts;",
                vec![Value::String(xpath.to_string())],
            )
            .await?;
        let texts: Vec<Vec<String>> = serde_json::from_value(value)?;
        Ok(texts
            .into_iter()
            .map(|texts| texts.iter().map(|text| normalize(text)).collect())
            .collect())
    }

    /// Replaces the value of a field the way a user does. WebDriver's `clear()` doesn't send
    /// an `input` event, so vertigo would keep the old value.
    pub async fn retype(&self, element: &WebElement, text: &str) -> Result<()> {
        element.click().await?;
        self.js_with("arguments[0].select()", vec![element.to_json()?])
            .await?;
        element.send_keys(Key::Backspace).await?;
        if !text.is_empty() {
            element.send_keys(text).await?;
        }
        Ok(())
    }

    pub async fn value(&self, element: &WebElement) -> Result<String> {
        Ok(element.prop("value").await?.unwrap_or_default())
    }

    /// Waits until the first match is a field with the `expected` value.
    pub async fn wait_for_value(&self, by: By, expected: &str) -> Result<()> {
        let mut last = None;
        self.wait_for(&format!("{by} with value {expected:?}"), async || {
            let value = self.value(&self.find(by.clone()).await?).await?;
            let matches = value == expected;
            last = Some(value);
            Ok(matches.then_some(()))
        })
        .await
        .with_context(|| format!("last value: {last:?}"))
    }

    /// Waits until the first match is shown or hidden (`display`, `visibility`, size).
    pub async fn wait_for_displayed(&self, by: By, displayed: bool) -> Result<()> {
        let state = if displayed { "shown" } else { "hidden" };
        self.wait_for(&format!("{by} to be {state}"), async || {
            let element = self.find(by.clone()).await?;
            Ok((element.is_displayed().await? == displayed).then_some(()))
        })
        .await
    }

    /// Computed style `property` (in CSS spelling, f. ex. `font-weight`) of `element`.
    pub async fn style(&self, element: &WebElement, property: &str) -> Result<String> {
        let value = self
            .js_with(
                "return getComputedStyle(arguments[0]).getPropertyValue(arguments[1])",
                vec![element.to_json()?, Value::String(property.to_string())],
            )
            .await?;
        value
            .as_str()
            .map(str::to_string)
            .with_context(|| format!("no style {property}: {value}"))
    }

    /// Path and query of the current URL.
    pub async fn path(&self) -> Result<String> {
        let url = self.driver.current_url().await?;
        Ok(match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_string(),
        })
    }

    pub async fn wait_for_path(&self, expected: &str) -> Result<()> {
        self.wait_for(&format!("the URL to change to {expected}"), async || {
            Ok((self.path().await? == expected).then_some(()))
        })
        .await
    }

    /// Marks the current document; `assert_same_document` then tells navigation inside the
    /// app from a full page load.
    pub async fn mark_document(&self) -> Result<()> {
        self.js("window.__e2eDocument = true").await?;
        Ok(())
    }

    pub async fn assert_same_document(&self) -> Result<()> {
        let same = self.js("return window.__e2eDocument === true").await?;
        ensure!(
            same == Value::Bool(true),
            "the browser loaded a new document instead of navigating in the app"
        );
        Ok(())
    }

    /// Console errors (SEVERE entries) containing `pattern` don't fail the test.
    pub fn allow_console(&self, pattern: &str) {
        self.allowed_console
            .lock()
            .expect("list of allowed console errors poisoned")
            .push(pattern.to_string());
    }

    /// Runs after the test body: fails the test on console errors and, on any failure, saves
    /// a screenshot, the page source and the console log.
    pub(crate) async fn finish(&self, result: Result<()>) -> Result<()> {
        let logs = self.driver.browser_log().await.unwrap_or_default();

        let result = result.and_then(|()| self.check_console(&logs));

        match result {
            Ok(()) => Ok(()),
            Err(err) => {
                let dir = self.save_artifacts(&err, &logs).await;
                Err(anyhow!(
                    "{err:?}\n\nscreenshot, page and console log: {}",
                    dir.display()
                ))
            }
        }
    }

    fn check_console(&self, logs: &[BrowserLogEntry]) -> Result<()> {
        let allowed = self
            .allowed_console
            .lock()
            .expect("list of allowed console errors poisoned");
        let errors: Vec<&str> = logs
            .iter()
            .filter(|entry| entry.level == "SEVERE")
            .map(|entry| entry.message.as_str())
            .filter(|message| !allowed.iter().any(|pattern| message.contains(pattern)))
            .collect();

        ensure!(
            errors.is_empty(),
            "the browser console reported errors ({}):\n  {}",
            errors.len(),
            errors.join("\n  ")
        );
        Ok(())
    }

    async fn save_artifacts(&self, err: &anyhow::Error, logs: &[BrowserLogEntry]) -> PathBuf {
        let dir = self
            .env
            .work_dir
            .join("artifacts")
            .join(self.name.replace("::", "__"));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);

        if let Ok(png) = self.driver.screenshot_as_png().await {
            let _ = std::fs::write(dir.join("screenshot.png"), png);
        }
        if let Ok(source) = self.driver.source().await {
            let _ = std::fs::write(dir.join("page.html"), source);
        }
        let url = match self.driver.current_url().await {
            Ok(url) => url.to_string(),
            Err(err) => format!("(unknown: {err})"),
        };
        let _ = std::fs::write(dir.join("error.txt"), format!("{url}\n\n{err:?}\n"));
        let console: String = logs
            .iter()
            .map(|entry| format!("{} {}\n", entry.level, entry.message))
            .collect();
        let _ = std::fs::write(dir.join("console.log"), console);

        dir
    }
}

/// Collapses whitespace, non-breaking spaces included.
pub fn normalize(text: &str) -> String {
    text.split(|c: char| c.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Quotes `text` as an XPath string literal.
pub fn xpath_literal(text: &str) -> String {
    if !text.contains('\'') {
        format!("'{text}'")
    } else if !text.contains('"') {
        format!("\"{text}\"")
    } else {
        let parts: Vec<String> = text.split('\'').map(|part| format!("'{part}'")).collect();
        format!("concat({})", parts.join(", \"'\", "))
    }
}
