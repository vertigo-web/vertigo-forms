//! Story "Select/Search": `SelectSearch` over a map of quotes, opening at 3 typed letters.

use vertigo_forms_e2e::prelude::*;

const INPUT: &str = "//input[@title='Enter phrase']";
/// Options in the drop-down, each with its key as `id`.
const OPTIONS: &str = "//input[@title='Enter phrase']/following-sibling::div/div";
const KEY: &str = "//p[starts-with(normalize-space(.), 'Selected key:')]";
const VALUE: &str = "//p[starts-with(normalize-space(.), 'Selected value:')]";

const ONCE: &str = "Once you choose hope, anything's possible";
const YOLO: &str = "You only live once but if you do it right, once is enough";

/// Background of the option chosen with the arrow keys (or under the mouse).
const HIGHLIGHT: &str = "rgb(204, 204, 204)";

async fn options(ctx: &Ctx) -> Result<Vec<String>> {
    ctx.texts(By::XPath(OPTIONS)).await
}

async fn wait_for_options(ctx: &Ctx, expected: &[&str]) -> Result<()> {
    let mut last = Vec::new();
    ctx.wait_for(&format!("options {expected:?}"), async || {
        let shown = options(ctx).await?;
        let matches = shown == expected;
        last = shown;
        Ok(matches.then_some(()))
    })
    .await
    .with_context(|| format!("last options: {last:?}"))
}

/// Waits until the option `key` is the highlighted one.
async fn wait_for_highlighted(ctx: &Ctx, key: &str) -> Result<()> {
    ctx.wait_for(&format!("option {key:?} to be highlighted"), async || {
        let mut highlighted = Vec::new();
        for option in ctx.find_all(By::XPath(OPTIONS)).await? {
            if ctx.style(&option, "background-color").await? == HIGHLIGHT {
                highlighted.push(option.attr("id").await?.unwrap_or_default());
            }
        }
        Ok((highlighted == [key]).then_some(()))
    })
    .await
}

async fn wait_for_selection(ctx: &Ctx, key: &str, value: &str) -> Result<()> {
    ctx.wait_for_text_of(By::XPath(KEY), &format!("Selected key: {key}"))
        .await?;
    ctx.wait_for_text_of(By::XPath(VALUE), &format!("Selected value: {value}"))
        .await?;
    ctx.wait_for_value(By::XPath(INPUT), value).await
}

pub async fn typing_filters_and_click_selects(ctx: Ctx) -> Result<()> {
    ctx.open("/select_search").await?;
    let hints = ctx
        .texts(By::XPath(
            "//h4[normalize-space(.)='Hints:']/following-sibling::ul/li",
        ))
        .await?;
    ensure!(
        hints == ["Once", "Start", "Try", "Well", "You"],
        "hints: {hints:?}"
    );

    let input = ctx.find(By::XPath(INPUT)).await?;
    input.send_keys("on").await?;
    ensure!(
        options(&ctx).await?.is_empty(),
        "options shown before 3 letters"
    );

    // Matched anywhere in the text, ignoring case, and sorted
    input.send_keys("ce").await?;
    wait_for_options(&ctx, &[ONCE, YOLO]).await?;

    ctx.find(By::Id("yolo")).await?.click().await?;
    wait_for_selection(&ctx, "yolo", YOLO).await?;
    ctx.wait_for_none(By::XPath(OPTIONS)).await
}

pub async fn arrows_and_enter_select(ctx: Ctx) -> Result<()> {
    ctx.open("/select_search").await?;
    let input = ctx.find(By::XPath(INPUT)).await?;
    input.send_keys("once").await?;
    wait_for_options(&ctx, &[ONCE, YOLO]).await?;

    input.send_keys(Key::Down).await?;
    wait_for_highlighted(&ctx, "once").await?;
    input.send_keys(Key::Down).await?;
    wait_for_highlighted(&ctx, "yolo").await?;
    input.send_keys(Key::Up).await?;
    wait_for_highlighted(&ctx, "once").await?;

    input.send_keys(Key::Enter).await?;
    wait_for_selection(&ctx, "once", ONCE).await?;
    ctx.wait_for_none(By::XPath(OPTIONS)).await
}

pub async fn leaving_field_closes_options(ctx: Ctx) -> Result<()> {
    ctx.open("/select_search").await?;
    let input = ctx.find(By::XPath(INPUT)).await?;
    input.send_keys("well").await?;
    wait_for_options(&ctx, &["Well done is better than well said"]).await?;

    ctx.by_text("h4", "Hints:").await?.click().await?;
    ctx.wait_for_none(By::XPath(OPTIONS)).await?;
    ctx.wait_for_text_of(By::XPath(KEY), "Selected key:").await
}
