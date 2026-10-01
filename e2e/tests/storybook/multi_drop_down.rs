//! Story "MultiDropDown": `MultiSelect` in a drop-down opened with a button.

use vertigo_forms_e2e::prelude::*;

use crate::multi_select::{SELECTED, wait_for_marked};

pub async fn drop_down_toggles_values(ctx: Ctx) -> Result<()> {
    let selected = By::XPath(SELECTED);
    let option = By::XPath("//button[normalize-space(.)='bar']");

    ctx.open("/multi_drop_down").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: foo")
        .await?;
    ensure!(
        ctx.find_all(option.clone()).await?.is_empty(),
        "the drop-down is open from the start"
    );

    ctx.click_button("V").await?;
    ctx.find(option.clone()).await?;
    ctx.button("^").await?;
    wait_for_marked(&ctx, &["foo"]).await?;

    ctx.click_button("bar").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: foo,bar")
        .await?;
    ctx.click_button("foo").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: bar")
        .await?;

    ctx.click_button("^").await?;
    ctx.wait_for_none(option).await?;
    ctx.button("V").await?;
    ctx.wait_for_text_of(selected, "Selected values: bar")
        .await?;

    // Opened again, it shows what is chosen
    ctx.click_button("V").await?;
    wait_for_marked(&ctx, &["bar"]).await
}
