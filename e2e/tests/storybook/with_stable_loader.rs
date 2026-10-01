//! Story "With Stable Loader": `WithStableLoader` and `WithLoader` side by side, of one
//! resource set with the buttons above them. Both render a field to type into.

use vertigo_forms_e2e::prelude::*;

const STABLE: &str = "WithStableLoader";
const PLAIN: &str = "WithLoader";

fn column(title: &str) -> String {
    format!("//div[h3[normalize-space(.)={}]]", xpath_literal(title))
}

fn field(title: &str) -> By {
    By::XPath(format!("{}//input", column(title)))
}

async fn wait_for_spinner(ctx: &Ctx, title: &str, expected: bool) -> Result<()> {
    let column = ctx.find(By::XPath(column(title))).await?;
    ctx.wait_for(&format!("spinner in {title}: {expected}"), async || {
        Ok((ctx.has_spinner(&column).await? == expected).then_some(()))
    })
    .await
}

async fn wait_for_ready(ctx: &Ctx, title: &str) -> Result<()> {
    ctx.wait_for_text_of(
        By::XPath(format!("{}//p[1]", column(title))),
        "Resource ready: Updated value",
    )
    .await?;
    wait_for_spinner(ctx, title, false).await
}

async fn wait_for_error(ctx: &Ctx, title: &str) -> Result<()> {
    ctx.wait_for_text_of(
        By::XPath(format!("{}/main", column(title))),
        "Deliberate error",
    )
    .await
}

/// Until the resource is ready for the first time, both loaders show what it is.
pub async fn both_follow_resource_until_ready(ctx: Ctx) -> Result<()> {
    ctx.open("/with_stable_loader").await?;
    for title in [STABLE, PLAIN] {
        wait_for_spinner(&ctx, title, true).await?;
    }

    ctx.click_button("Set error").await?;
    for title in [STABLE, PLAIN] {
        wait_for_error(&ctx, title).await?;
    }
    ctx.click_button("Set loading").await?;
    for title in [STABLE, PLAIN] {
        wait_for_spinner(&ctx, title, true).await?;
    }

    ctx.click_button("Set ready").await?;
    for title in [STABLE, PLAIN] {
        wait_for_ready(&ctx, title).await?;
        ctx.wait_for_value(field(title), "").await?;
    }
    Ok(())
}

/// Once ready, the stable loader keeps its content, and what was typed into it.
pub async fn stable_loader_keeps_typed_text(ctx: Ctx) -> Result<()> {
    ctx.open("/with_stable_loader").await?;
    ctx.click_button("Set ready").await?;
    for title in [STABLE, PLAIN] {
        wait_for_ready(&ctx, title).await?;
    }
    ctx.find(field(STABLE)).await?.send_keys("kept").await?;
    ctx.find(field(PLAIN)).await?.send_keys("lost").await?;

    ctx.click_button("Set loading").await?;
    wait_for_spinner(&ctx, PLAIN, true).await?;
    ctx.wait_for_none(field(PLAIN)).await?;
    wait_for_ready(&ctx, STABLE).await?;
    ctx.wait_for_value(field(STABLE), "kept").await?;

    ctx.click_button("Set ready").await?;
    wait_for_ready(&ctx, PLAIN).await?;
    ctx.wait_for_value(field(PLAIN), "").await?;
    ctx.wait_for_value(field(STABLE), "kept").await?;

    ctx.click_button("Set error").await?;
    wait_for_error(&ctx, PLAIN).await?;
    wait_for_ready(&ctx, STABLE).await?;
    ctx.wait_for_value(field(STABLE), "kept").await
}
