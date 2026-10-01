//! Story "Select": `Select` of strings.

use vertigo_forms_e2e::prelude::*;

pub async fn choosing_option_sets_value(ctx: Ctx) -> Result<()> {
    let selected = By::XPath("//p[starts-with(normalize-space(.), 'Selected value:')]");

    ctx.open("/select").await?;
    // No empty option, as the value is one of the options
    let options = ctx.texts(By::XPath("//select/option")).await?;
    ensure!(options == ["foo", "bar", "baz"], "options: {options:?}");
    let select = SelectElement::new(&ctx.find(By::Tag("select")).await?).await?;
    let current = select.first_selected_option().await?.text().await?;
    ensure!(current == "foo", "selected {current:?}");
    ctx.wait_for_text_of(selected.clone(), "Selected value: foo")
        .await?;

    select.select_by_exact_text("baz").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected value: baz")
        .await?;
    select.select_by_exact_text("bar").await?;
    ctx.wait_for_text_of(selected, "Selected value: bar").await
}
