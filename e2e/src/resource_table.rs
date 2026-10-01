//! Helpers for `ResourceTable`. The storybook has two of them, with the same items, so
//! everything here is scoped to one table, found by its title.

use anyhow::{Context, Result};
use thirtyfour::{By, WebElement};
use vertigo_testing::xpath_literal;

use crate::Ctx;

/// "Cancel" of the storybook's tables (`ResourceTableLabels::cancel`). Open forms are told by it,
/// together with the fields they have.
const CANCEL: &str = "Cancel";

/// Row in view mode: its last cell holds the `row-buttons` of vertigo-forms.
const VIEW_ROW: &str = "//div[div/div[@data-testid='row-buttons']]";

/// `ResourceTable` found by its title.
pub struct Table<'a> {
    ctx: &'a Ctx,
    /// XPath of the table's container.
    xpath: String,
}

pub trait TableExt {
    fn table(&self, title: &str) -> Table<'_>;
}

impl TableExt for Ctx {
    fn table(&self, title: &str) -> Table<'_> {
        Table {
            ctx: self,
            xpath: format!("//div[div/h2[normalize-space(.)={}]]", xpath_literal(title)),
        }
    }
}

impl Table<'_> {
    /// Header cells: the header is rendered right below the title bar.
    pub async fn header(&self) -> Result<Vec<String>> {
        let header = self
            .ctx
            .children_texts(&format!("{}/div[2]", self.xpath))
            .await?;
        header.into_iter().next().context("the table has no header")
    }

    /// Cells of the rows in view mode, without the last one, with the buttons.
    pub async fn rows(&self) -> Result<Vec<Vec<String>>> {
        let mut rows = self
            .ctx
            .children_texts(&format!("{}{VIEW_ROW}", self.xpath))
            .await?;
        for row in &mut rows {
            row.pop();
        }
        Ok(rows)
    }

    /// Waits until the rows in view mode are exactly `expected`. A row being saved has no
    /// buttons, so this also waits for saves to finish.
    pub async fn wait_for_rows(&self, expected: &[&[&str]]) -> Result<()> {
        let mut last = Vec::new();
        self.ctx
            .wait_for(&format!("rows {expected:?}"), async || {
                let rows = self.rows().await?;
                let matches = rows == expected;
                last = rows;
                Ok(matches.then_some(()))
            })
            .await
            .with_context(|| format!("last rows: {last:?}"))
    }

    /// Waits until the last row in view mode is `expected`.
    pub async fn wait_for_rows_ending(&self, expected: &[&str]) -> Result<()> {
        let mut last = None;
        self.ctx
            .wait_for(&format!("last row {expected:?}"), async || {
                let row = self.rows().await?.pop();
                let matches = row.as_ref().is_some_and(|row| *row == expected);
                last = row;
                Ok(matches.then_some(()))
            })
            .await
            .with_context(|| format!("last row: {last:?}"))
    }

    /// Clicks "Edit" or "Delete" in the row in view mode with a cell of exactly this text.
    pub async fn click_in_row(&self, cell: &str, label: &str) -> Result<()> {
        self.ctx
            .find(By::XPath(format!(
                "{}{VIEW_ROW}[div[normalize-space(.)={}]]//div[@data-testid='row-buttons']/div[normalize-space(.)={}]",
                self.xpath,
                xpath_literal(cell),
                xpath_literal(label),
            )))
            .await
            .with_context(|| format!("no row with {cell:?} and button {label:?}"))?
            .click()
            .await?;
        Ok(())
    }

    /// Clicks the button in the title bar which opens the form of a new item, and waits for
    /// the form. The form has its own button with the same label, hence this selector.
    pub async fn open_add_form(&self, label: &str) -> Result<WebElement> {
        self.ctx
            .find(By::XPath(format!(
                "{}/div/h2/following-sibling::div[not(*) and normalize-space(.)={}]",
                self.xpath,
                xpath_literal(label)
            )))
            .await?
            .click()
            .await?;
        self.form().await
    }

    fn forms_xpath(&self) -> String {
        format!(
            "{}//div[div/div/div[not(*) and normalize-space(.)={}] and (.//input or .//select)]",
            self.xpath,
            xpath_literal(CANCEL)
        )
    }

    /// Open forms, in page order: the form of a new item first, then the edited rows.
    pub async fn forms(&self) -> Result<Vec<WebElement>> {
        self.ctx.find_all(By::XPath(self.forms_xpath())).await
    }

    /// The first open form. Waits for it.
    pub async fn form(&self) -> Result<WebElement> {
        self.ctx
            .find(By::XPath(self.forms_xpath()))
            .await
            .context("no open form")
    }

    pub async fn wait_for_no_form(&self) -> Result<()> {
        self.ctx.wait_for_none(By::XPath(self.forms_xpath())).await
    }

    /// Clicks a button of the first open form: "Save", "Add Item" or "Cancel".
    pub async fn click_form_button(&self, label: &str) -> Result<()> {
        click_in_form(&self.form().await?, label).await
    }

    /// Waits for a cell or a message (an error, the question about deleting) of exactly this
    /// text in the table.
    pub async fn wait_for_text(&self, text: &str) -> Result<()> {
        self.ctx
            .find(self.text(text))
            .await
            .with_context(|| format!("no {text:?} in the table"))?;
        Ok(())
    }

    pub async fn wait_for_no_text(&self, text: &str) -> Result<()> {
        self.ctx.wait_for_none(self.text(text)).await
    }

    fn text(&self, text: &str) -> By {
        By::XPath(format!(
            "{}//div[not(*) and normalize-space(.)={}]",
            self.xpath,
            xpath_literal(text)
        ))
    }

    /// Answers the question about deleting a row: "Confirm" or "Cancel".
    pub async fn answer_delete(&self, question: &str, label: &str) -> Result<()> {
        self.ctx
            .find(By::XPath(format!(
                "{}//div[div[normalize-space(.)={}]]//div[not(*) and normalize-space(.)={}]",
                self.xpath,
                xpath_literal(question),
                xpath_literal(label)
            )))
            .await?
            .click()
            .await?;
        Ok(())
    }
}

/// Clicks a button of `form`: "Save", "Add Item" or "Cancel".
pub async fn click_in_form(form: &WebElement, label: &str) -> Result<()> {
    form.find(By::XPath(format!(
        ".//div[not(*) and normalize-space(.)={}]",
        xpath_literal(label)
    )))
    .await
    .with_context(|| format!("no button {label:?} in the form"))?
    .click()
    .await?;
    Ok(())
}
