//! Helpers for the storybook: its tabs, a story under each path, and what the stories are made
//! of.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{Value, json};
use thirtyfour::{By, WebElement};
use vertigo_testing::xpath_literal;

use crate::Ctx;

/// Background of the current tab's header, from `header_active_item_add_css` of the
/// storybook's `bordered_tabs()`, which all the tabs in it use.
const ACTIVE_TAB_BACKGROUND: &str = "rgb(211, 211, 211)";

/// A 1x1 picture to upload and drop.
pub const PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0xf7, 0x03, 0x41, 0x43, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
    0x44, 0xae, 0x42, 0x60, 0x82,
];

/// Helpers of the storybook's pages.
pub trait StorybookExt {
    async fn tab(&self, name: &str) -> Result<WebElement>;
    async fn click_tab(&self, name: &str) -> Result<()>;
    async fn wait_for_tab_active(&self, name: &str, active: bool) -> Result<()>;
    async fn button(&self, label: &str) -> Result<WebElement>;
    async fn click_button(&self, label: &str) -> Result<()>;
    async fn has_spinner(&self, container: &WebElement) -> Result<bool>;
    fn allow_sample_pictures(&self);
    fn picture_file(&self, name: &str) -> Result<PathBuf>;
    async fn drop_picture(&self, target: &WebElement, name: &str) -> Result<()>;
}

impl StorybookExt for Ctx {
    /// Header of the tab `name` (`Tabs`, `TabsHeader`): one of the storybook's own tabs, one
    /// per story, or of the tabs inside a story.
    async fn tab(&self, name: &str) -> Result<WebElement> {
        self.find(By::XPath(format!(
            "//li/a[normalize-space(.)={}]",
            xpath_literal(name)
        )))
        .await
    }

    async fn click_tab(&self, name: &str) -> Result<()> {
        self.tab(name).await?.click().await?;
        Ok(())
    }

    /// Waits until the header of tab `name` is styled as the current one, or as not current.
    /// The header is rendered anew on every change, so each check finds it again.
    async fn wait_for_tab_active(&self, name: &str, active: bool) -> Result<()> {
        let state = if active { "current" } else { "not current" };
        self.wait_for(&format!("tab {name:?} to be {state}"), async || {
            let tab = self.tab(name).await?;
            let background = self.style(&tab, "background-color").await?;
            Ok(((background == ACTIVE_TAB_BACKGROUND) == active).then_some(()))
        })
        .await
    }

    /// `<button>` with exactly this text.
    async fn button(&self, label: &str) -> Result<WebElement> {
        self.find(By::XPath(format!(
            "//button[normalize-space(.)={}]",
            xpath_literal(label)
        )))
        .await
    }

    async fn click_button(&self, label: &str) -> Result<()> {
        self.button(label).await?.click().await?;
        Ok(())
    }

    /// Whether there is a `Spinner` in `container`: a `div` animated forever.
    async fn has_spinner(&self, container: &WebElement) -> Result<bool> {
        let found = self
            .js_with(
                "return Array.from(arguments[0].querySelectorAll('div'))
                    .some(div => getComputedStyle(div).animationIterationCount === 'infinite')",
                vec![container.to_json()?],
            )
            .await?;
        Ok(found == Value::Bool(true))
    }

    /// The sample pictures of the stories come from picsum.photos, which the browser can't
    /// reach (see `TestEnv::start`).
    fn allow_sample_pictures(&self) {
        self.allow_console("https://picsum.photos/");
    }

    /// Writes [`PNG`] to a file named `name`, in a directory of this test, to upload it from
    /// there.
    fn picture_file(&self, name: &str) -> Result<PathBuf> {
        let dir = self
            .env
            .work_dir
            .join("files")
            .join(self.name.replace("::", "__"));
        std::fs::create_dir_all(&dir).with_context(|| format!("can't create {}", dir.display()))?;
        let path = dir.join(name);
        std::fs::write(&path, PNG).with_context(|| format!("can't write {}", path.display()))?;
        Ok(path)
    }

    /// Drops [`PNG`], as a file named `name`, on `target`, the way a file dragged from the
    /// desktop is dropped. WebDriver can't drag files, so the events come from a script.
    async fn drop_picture(&self, target: &WebElement, name: &str) -> Result<()> {
        self.js_with(
            "const [target, name, bytes] = arguments;
             const data = new DataTransfer();
             data.items.add(new File([new Uint8Array(bytes)], name, { type: 'image/png' }));
             for (const type of ['dragenter', 'dragover', 'drop']) {
                 target.dispatchEvent(new DragEvent(type, {
                     dataTransfer: data, bubbles: true, cancelable: true,
                 }));
             }",
            vec![target.to_json()?, json!(name), json!(PNG)],
        )
        .await?;
        Ok(())
    }
}
