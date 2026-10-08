//! Story "Popup": `Popup` shown by a `Switch`, and `PopupOnHover`.

use vertigo_forms_e2e::prelude::*;

fn paragraph(text: &str) -> By {
    By::XPath(format!("//p[normalize-space(.)={}]", xpath_literal(text)))
}

pub async fn switch_shows_and_hides_popup(ctx: Ctx) -> Result<()> {
    let content = paragraph("Content in the popup");

    ctx.open("/popup").await?;
    ctx.wait_for_displayed(content.clone(), false).await?;

    ctx.click_button("OFF").await?;
    ctx.wait_for_displayed(content.clone(), true).await?;
    ctx.click_button("ON").await?;
    ctx.wait_for_displayed(content, false).await
}

pub async fn hover_shows_popup(ctx: Ctx) -> Result<()> {
    let content = paragraph("Hover popup content");

    ctx.open("/popup").await?;
    ctx.wait_for_displayed(content.clone(), false).await?;

    let element = ctx.find(paragraph("Popup on hover")).await?;
    ctx.action_chain()
        .move_to_element_center(&element)
        .perform()
        .await?;
    ctx.wait_for_displayed(content.clone(), true).await?;

    let elsewhere = ctx.tab("Popup").await?;
    ctx.action_chain()
        .move_to_element_center(&elsewhere)
        .perform()
        .await?;
    ctx.wait_for_displayed(content, false).await
}
