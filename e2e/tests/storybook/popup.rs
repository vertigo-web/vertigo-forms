//! Story "Popup": `Popup` shown by a `Switch`, and `PopupOnHover`.

use vertigo_forms_e2e::prelude::*;

fn paragraph(text: &str) -> By {
    By::XPath(format!("//p[normalize-space(.)={}]", xpath_literal(text)))
}

/// Hovers over `target` until `content` is shown. With `E2E_HEADLESS=0` a hover doesn't always
/// last: when a window of a parallel test opens or closes under the system pointer, Chrome takes
/// the pointer for gone from the page, or moved to where the system one is, and `:hover` goes.
/// So each check moves the pointer there again.
async fn hover_until_shown(ctx: &Ctx, target: By, content: By) -> Result<()> {
    ctx.wait_for(&format!("{content} to be shown on hover"), async || {
        let element = ctx.find(target.clone()).await?;
        ctx.action_chain()
            .move_to_element_center(&element)
            .perform()
            .await?;
        let shown = ctx.find(content.clone()).await?.is_displayed().await?;
        Ok(shown.then_some(()))
    })
    .await
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

    hover_until_shown(&ctx, paragraph("Popup on hover"), content.clone()).await?;

    let elsewhere = ctx.tab("Popup").await?;
    ctx.action_chain()
        .move_to_element_center(&elsewhere)
        .perform()
        .await?;
    ctx.wait_for_displayed(content, false).await
}
