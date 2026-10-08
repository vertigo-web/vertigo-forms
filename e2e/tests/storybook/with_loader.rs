//! Story "With Loader": `WithLoader` of a resource set with the buttons above it.

use vertigo_forms_e2e::prelude::*;

const STORY: &str = "//div[p/button[normalize-space(.)='Set loading']]";

pub async fn follows_resource(ctx: Ctx) -> Result<()> {
    let main = By::XPath(format!("{STORY}/main"));

    ctx.open("/with_loader").await?;
    let story = ctx.find(By::XPath(STORY)).await?;
    ctx.wait_for_text_of(main.clone(), "Resource ready: Initial value")
        .await?;
    ensure!(!ctx.has_spinner(&story).await?, "spinner next to the value");

    ctx.click_button("Set loading").await?;
    ctx.wait_for_none(main.clone()).await?;
    ensure!(ctx.has_spinner(&story).await?, "no spinner while loading");

    ctx.click_button("Set ready").await?;
    ctx.wait_for_text_of(main.clone(), "Resource ready: Updated value")
        .await?;
    ensure!(!ctx.has_spinner(&story).await?, "spinner next to the value");

    ctx.click_button("Set error").await?;
    ctx.wait_for_text_of(main, "Deliberate error").await?;
    ensure!(!ctx.has_spinner(&story).await?, "spinner next to the error");
    Ok(())
}
