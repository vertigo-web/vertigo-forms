//! Story "Tooltip": a word and a sign at the right edge, each with a hint, and a button with
//! a hint, which a click and a tap still press.

use thirtyfour::common::command::{Actions, Command};
use vertigo_forms_e2e::prelude::*;

const WORD: &str = "cat";
const WORD_HINT: &str = "Clearly not a dog";
const SIGN: &str = "★ 4.5";
const SIGN_HINT: &str =
    "4.5 out of 5 stars from 120 reviews, most of them praising the friendly service";
const BUTTON_HINT: &str = "The cat had its breakfast an hour ago";

fn button() -> By {
    By::XPath("//button[normalize-space(.)='Feed the cat']".to_string())
}

/// Waits until the story says the button fed the cat `times` times.
async fn wait_fed(ctx: &Ctx, times: u32) -> Result<()> {
    let expected = format!("Fed {times} times");
    ctx.wait_for(&format!("{expected:?}"), async || {
        let text = ctx
            .find(By::XPath(
                "//span[starts-with(normalize-space(.), 'Fed ')]".to_string(),
            ))
            .await?
            .text()
            .await?;
        Ok((text == expected).then_some(()))
    })
    .await
}

/// The element with the hint, by its text.
fn element(text: &str) -> By {
    By::XPath(format!(
        "//span[@tabindex='0'][*[1][normalize-space(.)={}]]",
        xpath_literal(text)
    ))
}

fn bubble(text: &str) -> By {
    By::XPath(format!(
        "//span[@role='tooltip'][normalize-space(.)={}]",
        xpath_literal(text)
    ))
}

/// Whether the hint is in sight: hidden, it stays in the page a pixel big, out of sight.
async fn in_sight(ctx: &Ctx, text: &str) -> Result<bool> {
    let rect = ctx.find(bubble(text)).await?.rect().await?;
    Ok(rect.width > 2.0 && rect.height > 2.0)
}

async fn wait_in_sight(ctx: &Ctx, text: &str, expected: bool) -> Result<()> {
    let what = if expected { "in sight" } else { "out of sight" };
    ctx.wait_for(&format!("{text:?} {what}"), async || {
        Ok((in_sight(ctx, text).await? == expected).then_some(()))
    })
    .await
}

/// Taps `target` with a finger.
async fn tap(ctx: &Ctx, target: &WebElement) -> Result<()> {
    let actions = json!([{
        "type": "pointer",
        "id": "finger",
        "parameters": { "pointerType": "touch" },
        "actions": [
            { "type": "pointerMove", "origin": target.to_json()?, "x": 0, "y": 0, "duration": 0 },
            { "type": "pointerDown", "button": 0 },
            { "type": "pointerUp", "button": 0 },
        ],
    }]);
    ctx.cmd(Command::PerformActions(Actions::from(actions)))
        .await?;
    Ok(())
}

/// A mouse over the word shows its hint, under it, and leaving the word hides it.
pub async fn hover_shows_hint_and_leaving_hides_it(ctx: Ctx) -> Result<()> {
    ctx.open("/tooltip").await?;
    wait_in_sight(&ctx, WORD_HINT, false).await?;

    // With `E2E_HEADLESS=0` a hover doesn't always last (see the story "Popup"), so it's repeated
    ctx.wait_for("the hint on hover", async || {
        let word = ctx.find(element(WORD)).await?;
        ctx.action_chain()
            .move_to_element_center(&word)
            .perform()
            .await?;
        Ok(in_sight(&ctx, WORD_HINT).await?.then_some(()))
    })
    .await?;
    let word = ctx.find(element(WORD)).await?.rect().await?;
    let hint = ctx.find(bubble(WORD_HINT)).await?.rect().await?;
    ensure!(
        hint.y >= word.y + word.height,
        "the hint isn't under the word"
    );

    let elsewhere = ctx.tab("Tooltip").await?;
    ctx.action_chain()
        .move_to_element_center(&elsewhere)
        .perform()
        .await?;
    wait_in_sight(&ctx, WORD_HINT, false).await
}

/// A tap shows the hint, and a tap elsewhere hides it; a second tap on the word too.
pub async fn tap_shows_hint_and_tap_elsewhere_hides_it(ctx: Ctx) -> Result<()> {
    ctx.open("/tooltip").await?;
    let word = ctx.find(element(WORD)).await?;
    tap(&ctx, &word).await?;
    wait_in_sight(&ctx, WORD_HINT, true).await?;

    let elsewhere = ctx.find(element(SIGN)).await?;
    tap(&ctx, &elsewhere).await?;
    wait_in_sight(&ctx, WORD_HINT, false).await?;
    // The sign tapped shows its own
    wait_in_sight(&ctx, SIGN_HINT, true).await?;
    tap(&ctx, &elsewhere).await?;
    wait_in_sight(&ctx, SIGN_HINT, false).await
}

/// Tab brings the focus to the word and shows its hint; Escape hides it, Enter shows it again,
/// and Tab to the next element hides it and shows the next hint.
pub async fn tab_shows_hint_and_escape_hides_it(ctx: Ctx) -> Result<()> {
    ctx.open("/tooltip").await?;
    ctx.find(element(WORD)).await?.focus().await?;
    wait_in_sight(&ctx, WORD_HINT, true).await?;
    ctx.action_chain().send_keys(Key::Escape).perform().await?;
    wait_in_sight(&ctx, WORD_HINT, false).await?;
    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_in_sight(&ctx, WORD_HINT, true).await?;
    ctx.action_chain().send_keys(Key::Tab).perform().await?;
    wait_in_sight(&ctx, WORD_HINT, false).await?;
    wait_in_sight(&ctx, SIGN_HINT, true).await
}

/// The hint of the sign at the right edge stays within the window, 8 pixels from its edge.
pub async fn hint_keeps_within_the_window(ctx: Ctx) -> Result<()> {
    ctx.open("/tooltip").await?;
    ctx.find(element(SIGN)).await?.focus().await?;
    wait_in_sight(&ctx, SIGN_HINT, true).await?;
    let width = ctx
        .js("return document.documentElement.clientWidth")
        .await?
        .as_f64()
        .context("the width of the window")?;
    let hint = ctx.find(bubble(SIGN_HINT)).await?.rect().await?;
    ensure!(
        hint.x >= 7.5 && hint.x + hint.width <= width - 7.5,
        "the hint at {} wide {} sticks out of the window {width} wide",
        hint.x,
        hint.width
    );
    Ok(())
}

/// On a button the hint shows under the mouse, and a click does what the button does and hides
/// the hint till the mouse leaves; a tap does it too, without the hint, and the button is the only
/// tab stop, so Enter presses it.
pub async fn a_button_keeps_its_click_and_shows_its_hint_on_hover(ctx: Ctx) -> Result<()> {
    ctx.open("/tooltip").await?;
    ctx.wait_for("the hint over the button", async || {
        let button = ctx.find(button()).await?;
        ctx.action_chain()
            .move_to_element_center(&button)
            .perform()
            .await?;
        Ok(in_sight(&ctx, BUTTON_HINT).await?.then_some(()))
    })
    .await?;
    ctx.find(button()).await?.click().await?;
    wait_fed(&ctx, 1).await?;
    wait_in_sight(&ctx, BUTTON_HINT, false).await?;

    let button = ctx.find(button()).await?;
    tap(&ctx, &button).await?;
    wait_fed(&ctx, 2).await?;
    ensure!(
        !in_sight(&ctx, BUTTON_HINT).await?,
        "a tap on the button showed its hint"
    );

    let tab_stops = ctx
        .js("return document.querySelectorAll('span[tabindex] > button').length")
        .await?;
    ensure!(
        tab_stops.as_u64() == Some(0),
        "the button's hint is a tab stop of its own"
    );
    button.focus().await?;
    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_fed(&ctx, 3).await
}
