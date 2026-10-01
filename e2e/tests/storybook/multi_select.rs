//! Story "MultiSelect": `MultiSelect` of strings, a button for each option.

use vertigo_forms_e2e::prelude::*;

pub const SELECTED: &str = "//p[starts-with(normalize-space(.), 'Selected values:')]";

/// Waits until exactly the `selected` options are marked as chosen (in bold).
pub async fn wait_for_marked(ctx: &Ctx, selected: &[&str]) -> Result<()> {
    ctx.wait_for(&format!("options {selected:?} to be marked"), async || {
        for option in ["foo", "bar", "baz"] {
            let button = ctx.button(option).await?;
            let bold = ctx.style(&button, "font-weight").await? == "700";
            if bold != selected.contains(&option) {
                return Ok(None);
            }
        }
        Ok(Some(()))
    })
    .await
}

pub async fn buttons_toggle_values(ctx: Ctx) -> Result<()> {
    let selected = By::XPath(SELECTED);

    ctx.open("/multi_select").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: foo")
        .await?;
    wait_for_marked(&ctx, &["foo"]).await?;

    ctx.click_button("bar").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: foo,bar")
        .await?;
    ctx.click_button("foo").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: bar")
        .await?;
    ctx.click_button("baz").await?;
    ctx.wait_for_text_of(selected, "Selected values: bar,baz")
        .await?;
    wait_for_marked(&ctx, &["bar", "baz"]).await
}
