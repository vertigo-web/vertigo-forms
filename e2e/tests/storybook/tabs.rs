//! Story "Tabs": `TabsHeader` and `TabsContentMapped`, which shows a tab for each of its sub
//! views too.

use vertigo_forms_e2e::prelude::*;

/// Content of the story's tabs, next to their header.
const CONTENT: &str = "//ul[li/a[normalize-space(.)='View 1']]/following-sibling::div[1]";

async fn wait_for_view(ctx: &Ctx, content: &str, current: &str) -> Result<()> {
    ctx.wait_for_text_of(By::XPath(format!("{CONTENT}/p[1]")), content)
        .await?;
    ctx.wait_for_text_of(
        By::XPath(format!("{CONTENT}/p[2]")),
        &format!("Current view: {current}"),
    )
    .await
}

pub async fn header_switches_tabs(ctx: Ctx) -> Result<()> {
    ctx.open("/tabs").await?;
    wait_for_view(&ctx, "View 1 content", "View1").await?;
    ctx.wait_for_tab_active("View 1", true).await?;
    ctx.wait_for_tab_active("View 2", false).await?;

    ctx.click_tab("View 2").await?;
    wait_for_view(&ctx, "View 2 content", "View2SubView1").await?;
    ctx.wait_for_tab_active("View 2", true).await?;
    ctx.wait_for_tab_active("View 1", false).await?;

    ctx.click_tab("View 1").await?;
    wait_for_view(&ctx, "View 1 content", "View1").await?;
    ctx.wait_for_tab_active("View 1", true).await?;
    ctx.wait_for_tab_active("View 2", false).await
}

pub async fn sub_views_show_their_tab(ctx: Ctx) -> Result<()> {
    ctx.open("/tabs").await?;

    ctx.click_button("Go to sub view 1.2").await?;
    wait_for_view(&ctx, "View 1 content", "View1SubView2").await?;
    ctx.click_button("Go to sub view 1.1").await?;
    wait_for_view(&ctx, "View 1 content", "View1SubView1").await?;

    ctx.click_tab("View 2").await?;
    ctx.click_button("Go to sub view 2.2").await?;
    wait_for_view(&ctx, "View 2 content", "View2SubView2").await?;

    ctx.click_button("Go back to view 1").await?;
    wait_for_view(&ctx, "View 1 content", "View1").await?;
    ctx.wait_for_tab_active("View 1", true).await
}
