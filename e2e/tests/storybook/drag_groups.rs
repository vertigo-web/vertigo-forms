//! Story "Drag Groups": chores in three groups, "To do", "Doing" and "Done", moved between them
//! at once.

use thirtyfour::common::command::{Actions, Command};
use vertigo_forms_e2e::prelude::*;

const SUMMARY: &str = "//p[starts-with(normalize-space(.), 'Groups:')]";

fn group(name: &str) -> By {
    By::XPath(format!(
        "//div[@role='group'][@aria-label={}]",
        xpath_literal(name)
    ))
}

fn item(group_name: &str, label: &str) -> By {
    By::XPath(format!(
        "//div[@role='group'][@aria-label={}]//button[normalize-space(.)={}]",
        xpath_literal(group_name),
        xpath_literal(label)
    ))
}

/// The "Here" buttons, in the groups the picked chore may go to.
fn here() -> By {
    By::XPath("//div[@role='group']//button[normalize-space(.)='Here']".to_string())
}

fn here_in(group_name: &str) -> By {
    By::XPath(format!(
        "//div[@role='group'][@aria-label={}]//button[normalize-space(.)='Here']",
        xpath_literal(group_name)
    ))
}

async fn open_story(ctx: &Ctx) -> Result<()> {
    ctx.open("/drag_groups").await?;
    wait_for_groups(ctx, "To do: Wash, Cook, Sweep; Doing: Shop; Done: nothing").await
}

async fn wait_for_groups(ctx: &Ctx, expected: &str) -> Result<()> {
    ctx.wait_for_text_of(
        By::XPath(SUMMARY.to_string()),
        &format!("Groups: {expected}"),
    )
    .await
}

async fn wait_for_picked(ctx: &Ctx, group_name: &str, label: &str, picked: bool) -> Result<()> {
    let expected = picked.to_string();
    ctx.wait_for(&format!("{label:?} to be picked: {picked}"), async || {
        let button = ctx.find(item(group_name, label)).await?;
        Ok((button.attr("aria-pressed").await?.as_deref() == Some(&expected)).then_some(()))
    })
    .await
}

/// Touches `from` with a finger in its center, moves the finger to the center of `to` and lifts
/// it there, as on a touch screen.
async fn drag_with_finger(ctx: &Ctx, from: &WebElement, to: &WebElement) -> Result<()> {
    let actions = json!([{
        "type": "pointer",
        "id": "finger",
        "parameters": { "pointerType": "touch" },
        "actions": [
            { "type": "pointerMove", "origin": from.to_json()?, "x": 0, "y": 0, "duration": 0 },
            { "type": "pointerDown", "button": 0 },
            { "type": "pointerMove", "origin": to.to_json()?, "x": 0, "y": 0, "duration": 200 },
            { "type": "pointerUp", "button": 0 },
        ],
    }]);
    ctx.cmd(Command::PerformActions(Actions::from(actions)))
        .await?;
    Ok(())
}

pub async fn mouse_drags_item_to_another_group(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    let from = ctx.find(item("To do", "Cook")).await?;
    let to = ctx.find(group("Done")).await?;
    ctx.action_chain()
        .click_and_hold_element(&from)
        .move_to_element_center(&to)
        .release()
        .perform()
        .await?;

    wait_for_groups(&ctx, "To do: Wash, Sweep; Doing: Shop; Done: Cook").await?;
    ctx.wait_for_none(here()).await
}

/// The browser captures a finger for the chore it touched. The chore releases it, so the finger
/// is lifted over the group it goes to, which takes the chore.
pub async fn finger_drags_item_to_another_group(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    let from = ctx.find(item("To do", "Sweep")).await?;
    let to = ctx.find(group("Doing")).await?;
    drag_with_finger(&ctx, &from, &to).await?;

    wait_for_groups(&ctx, "To do: Wash, Cook; Doing: Shop, Sweep; Done: nothing").await?;
    ctx.wait_for_none(here()).await
}

/// A tap picks a chore, and "Here" shows in every other group. A tap on one of them puts the
/// chore there.
pub async fn click_picks_and_here_puts(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    ctx.find(item("Doing", "Shop")).await?.click().await?;
    wait_for_picked(&ctx, "Doing", "Shop", true).await?;
    let groups = ctx.wait_for("Here in the other groups", async || {
        let mut groups = Vec::new();
        for name in ["To do", "Doing", "Done"] {
            if !ctx.find_all(here_in(name)).await?.is_empty() {
                groups.push(name);
            }
        }
        Ok((groups.len() == 2).then_some(groups))
    });
    ensure!(groups.await? == ["To do", "Done"], "Here in its own group");
    // Nothing moved yet
    wait_for_groups(&ctx, "To do: Wash, Cook, Sweep; Doing: Shop; Done: nothing").await?;

    ctx.find(here_in("Done")).await?.click().await?;
    wait_for_groups(&ctx, "To do: Wash, Cook, Sweep; Doing: nothing; Done: Shop").await?;
    ctx.wait_for_none(here()).await?;
    ctx.find(By::XPath(
        "//div[@aria-label='Doing']//span[normalize-space(.)='Nothing']".to_string(),
    ))
    .await
    .context("an empty group says so")?;
    Ok(())
}

/// A second click on the picked chore puts it down, and so does Escape.
pub async fn second_click_or_escape_puts_down(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    let wash = ctx.find(item("To do", "Wash")).await?;
    wash.click().await?;
    wait_for_picked(&ctx, "To do", "Wash", true).await?;
    wash.click().await?;
    wait_for_picked(&ctx, "To do", "Wash", false).await?;
    ctx.wait_for_none(here()).await?;

    wash.click().await?;
    ctx.find(here()).await?;
    ctx.action_chain().send_keys(Key::Escape).perform().await?;
    wait_for_picked(&ctx, "To do", "Wash", false).await?;
    ctx.wait_for_none(here()).await
}

/// Enter picks the chore in focus, and Enter on "Here" puts it there.
pub async fn keyboard_picks_and_puts(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    ctx.find(item("To do", "Wash")).await?.focus().await?;
    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_for_picked(&ctx, "To do", "Wash", true).await?;

    ctx.find(here_in("Doing")).await?.focus().await?;
    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_for_groups(&ctx, "To do: Cook, Sweep; Doing: Shop, Wash; Done: nothing").await
}
