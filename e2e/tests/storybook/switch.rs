//! Story "Switch": two `Switch` buttons of one value, with the default symbols and with
//! custom ones.

use vertigo_forms_e2e::prelude::*;

/// Custom symbols of the second switch: Greek capital letters, not Latin "O".
const THETA: &str = "\u{398}";
const OMICRON: &str = "\u{39f}";

fn line(n: u8) -> By {
    By::XPath(format!(
        "//p[starts-with(normalize-space(.), 'Toggle {n}:')]"
    ))
}

async fn wait_for_lines(ctx: &Ctx, on: bool) -> Result<()> {
    let (symbol, custom) = if on { ("ON", THETA) } else { ("OFF", OMICRON) };
    ctx.wait_for_text_of(line(1), &format!("Toggle 1: {symbol} {on}"))
        .await?;
    ctx.wait_for_text_of(line(2), &format!("Toggle 2: {custom} {on}"))
        .await
}

pub async fn switches_share_the_value(ctx: Ctx) -> Result<()> {
    ctx.open("/switch").await?;
    wait_for_lines(&ctx, false).await?;

    ctx.click_button("OFF").await?;
    wait_for_lines(&ctx, true).await?;
    ctx.click_button(THETA).await?;
    wait_for_lines(&ctx, false).await?;
    ctx.click_button(OMICRON).await?;
    wait_for_lines(&ctx, true).await
}
