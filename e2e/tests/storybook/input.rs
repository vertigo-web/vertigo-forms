//! Story "Input": `Input`, `InputWithButton` and `ListInput`, each with its own value shown
//! below it.

use vertigo_forms_e2e::prelude::*;

/// The field of the section titled `title`.
fn field(title: &str) -> By {
    By::XPath(format!(
        "//h4[normalize-space(.)={}]/following-sibling::p[1]/input",
        xpath_literal(title)
    ))
}

/// "Entered value: ..." of the section titled `title`.
fn entered(title: &str) -> By {
    By::XPath(format!(
        "//h4[normalize-space(.)={}]/following-sibling::p[2]",
        xpath_literal(title)
    ))
}

pub async fn input_sets_value_as_typed(ctx: Ctx) -> Result<()> {
    ctx.open("/input").await?;
    let input = ctx.find(field("Input")).await?;

    input.send_keys("hello").await?;
    ctx.wait_for_text_of(entered("Input"), "Entered value: hello")
        .await?;
    ctx.retype(&input, "bye").await?;
    ctx.wait_for_text_of(entered("Input"), "Entered value: bye")
        .await?;
    ctx.retype(&input, "").await?;
    ctx.wait_for_text_of(entered("Input"), "Entered value:")
        .await?;

    for other in ["InputWithButton", "ListInput"] {
        ensure!(
            ctx.text_of(entered(other)).await? == "Entered value:",
            "{other} got a value too"
        );
    }
    Ok(())
}

pub async fn input_with_button_sets_value_on_ok(ctx: Ctx) -> Result<()> {
    ctx.open("/input").await?;
    let input = ctx.find(field("InputWithButton")).await?;

    input.send_keys("draft").await?;
    ensure!(
        ctx.text_of(entered("InputWithButton")).await? == "Entered value:",
        "typing set the value before OK"
    );
    ctx.click_button("OK").await?;
    ctx.wait_for_text_of(entered("InputWithButton"), "Entered value: draft")
        .await?;
    ensure!(ctx.value(&input).await? == "draft", "the field was cleared");

    input.send_keys(" two").await?;
    ensure!(
        ctx.text_of(entered("InputWithButton")).await? == "Entered value: draft",
        "typing changed the value before OK"
    );
    ctx.click_button("OK").await?;
    ctx.wait_for_text_of(entered("InputWithButton"), "Entered value: draft two")
        .await
}

pub async fn list_input_splits_on_commas(ctx: Ctx) -> Result<()> {
    ctx.open("/input").await?;
    let input = ctx.find(field("ListInput")).await?;

    // Blanks around the items and empty items are dropped
    input.send_keys("a, b,,c ").await?;
    ctx.wait_for_text_of(entered("ListInput"), "Entered value: a :: b :: c")
        .await?;
    ctx.retype(&input, "").await?;
    ctx.wait_for_text_of(entered("ListInput"), "Entered value:")
        .await
}
