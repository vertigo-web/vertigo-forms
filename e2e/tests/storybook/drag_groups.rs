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

/// The copy of a chore which follows the pointer while it's dragged, outside of the groups.
fn copy_of(label: &str) -> By {
    By::XPath(format!(
        "//button[@aria-hidden='true'][normalize-space(.)={}]",
        xpath_literal(label)
    ))
}

/// The center of `element` on the page.
async fn center(element: &WebElement) -> Result<(f64, f64)> {
    let rect = element.rect().await?;
    Ok((rect.x + rect.width / 2.0, rect.y + rect.height / 2.0))
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

/// While a chore is dragged, a copy of it follows the pointer, held where it was grabbed. Let go
/// outside of the groups, the copy goes away and the chore stays where it was, picked.
pub async fn dragged_item_follows_the_pointer(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    let from = ctx.find(item("To do", "Cook")).await?;
    let to = ctx.find(group("Done")).await?;
    ctx.action_chain()
        .click_and_hold_element(&from)
        .move_to_element_center(&to)
        .perform()
        .await?;

    // Grabbed in its center, the copy stands with its center under the pointer
    let (x, y) = center(&ctx.find(copy_of("Cook")).await?).await?;
    let (to_x, to_y) = center(&to).await?;
    ensure!(
        (x - to_x).abs() < 2.0 && (y - to_y).abs() < 2.0,
        "the copy is centered at ({x}, {y}), the pointer at ({to_x}, {to_y})"
    );

    let outside = ctx.find(By::XPath(SUMMARY.to_string())).await?;
    ctx.action_chain()
        .move_to_element_center(&outside)
        .release()
        .perform()
        .await?;

    ctx.wait_for_none(copy_of("Cook")).await?;
    wait_for_picked(&ctx, "To do", "Cook", true).await?;
    wait_for_groups(&ctx, "To do: Wash, Cook, Sweep; Doing: Shop; Done: nothing").await
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

/// A chore picked with a tap can still be dragged - here with a finger - to another group.
pub async fn picked_item_can_still_be_dragged(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    let wash = ctx.find(item("To do", "Wash")).await?;
    wash.click().await?;
    wait_for_picked(&ctx, "To do", "Wash", true).await?;

    let to = ctx.find(group("Doing")).await?;
    drag_with_finger(&ctx, &wash, &to).await?;

    wait_for_groups(&ctx, "To do: Cook, Sweep; Doing: Shop, Wash; Done: nothing").await?;
    ctx.wait_for_none(here()).await
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
    wait_for_groups(&ctx, "To do: Cook, Sweep; Doing: Shop, Wash; Done: nothing").await?;
    // "Here" is gone with the picked chore, and the focus went along with the chore
    wait_for_focus(&ctx, "Doing/Wash").await
}

/// Escape on "Here" puts the picked chore down, and "Here" goes away: the focus goes back to the
/// chore.
pub async fn escape_on_here_gives_the_focus_back(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    ctx.find(item("To do", "Wash")).await?.focus().await?;
    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_for_picked(&ctx, "To do", "Wash", true).await?;

    ctx.find(here_in("Done")).await?.focus().await?;
    ctx.action_chain().send_keys(Key::Escape).perform().await?;

    wait_for_picked(&ctx, "To do", "Wash", false).await?;
    ctx.wait_for_none(here()).await?;
    wait_for_focus(&ctx, "To do/Wash").await
}

/// The group and the name of what has the focus, as "group/name": its label, or its text.
async fn focused(ctx: &Ctx) -> Result<String> {
    let found = ctx
        .js("const e = document.activeElement; \
             const group = e.closest(\"[role='group']\"); \
             const name = e.getAttribute('aria-label') || e.textContent.trim(); \
             return (group ? group.getAttribute('aria-label') : '') + '/' + name")
        .await?;
    Ok(found.as_str().unwrap_or_default().to_string())
}

async fn wait_for_focus(ctx: &Ctx, expected: &str) -> Result<()> {
    ctx.wait_for(&format!("the focus on {expected:?}"), async || {
        Ok((focused(ctx).await? == expected).then_some(()))
    })
    .await
}

/// The pin of a chore, at its end beside its button.
fn pin(label: &str) -> By {
    By::XPath(format!(
        "//button[@aria-label={}]",
        xpath_literal(&format!("Pin {label}"))
    ))
}

async fn opacity(element: &WebElement) -> Result<String> {
    Ok(element.css_value("opacity").await?)
}

/// Moves the mouse over the chore `label` of `group_name` till its pin shows. With
/// `E2E_HEADLESS=0` a hover doesn't always last (see the story "Popup"), so it's repeated.
async fn hover_till_pin_shows(ctx: &Ctx, group_name: &str, label: &str) -> Result<WebElement> {
    let pin = ctx.find(pin(label)).await?;
    ctx.wait_for(&format!("the pin of {label:?} to show"), async || {
        let chore = ctx.find(item(group_name, label)).await?;
        ctx.action_chain()
            .move_to_element_center(&chore)
            .perform()
            .await?;
        Ok((opacity(&pin).await? == "1").then_some(()))
    })
    .await?;
    Ok(pin)
}

async fn wait_for_pinned(ctx: &Ctx, expected: &str) -> Result<()> {
    ctx.wait_for_text_of(
        By::XPath("//p[starts-with(., 'Pinned:')]".to_string()),
        &format!("Pinned: {expected}"),
    )
    .await
}

/// What stands at the end of an item shows with the pointer over the item, and a click on it
/// does what it does without picking the item.
pub async fn aside_shows_over_the_item_and_does_not_pick_it(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    ensure!(
        opacity(&ctx.find(pin("Wash")).await?).await? == "0",
        "the pin hides at first"
    );
    let wash_pin = hover_till_pin_shows(&ctx, "To do", "Wash").await?;
    wash_pin.click().await?;
    wait_for_pinned(&ctx, "Wash").await?;
    wait_for_picked(&ctx, "To do", "Wash", false).await?;

    // Over the end of the chore, in the room it keeps for the pin, beside its name
    let wash = ctx.find(item("To do", "Wash")).await?;
    let (chore, name) = (
        wash.rect().await?,
        wash.find(By::Tag("span")).await?.rect().await?,
    );
    let pin_at = wash_pin.rect().await?;
    ensure!(
        (pin_at.x + pin_at.width - (chore.x + chore.width)).abs() < 1.0,
        "the pin ends at {}, the chore at {}",
        pin_at.x + pin_at.width,
        chore.x + chore.width
    );
    ensure!(
        name.x + name.width <= pin_at.x,
        "the pin, from {}, covers the name, to {}",
        pin_at.x,
        name.x + name.width
    );

    // Away from the item a pinned chore keeps its pin in sight
    ctx.action_chain()
        .move_to_element_center(&ctx.find(By::XPath(SUMMARY.to_string())).await?)
        .perform()
        .await?;
    ensure!(
        opacity(&wash_pin).await? == "1",
        "the pinned chore shows its pin"
    );
    let cook_pin = ctx.find(pin("Cook")).await?;
    ensure!(opacity(&cook_pin).await? == "0", "the others hide theirs");
    Ok(())
}

/// While a chore of another group is picked, a click on what stands at the end of an item does
/// what it does, and the picked chore stays where it is, picked.
pub async fn aside_click_leaves_the_picked_item_where_it_is(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    ctx.find(item("Doing", "Shop")).await?.click().await?;
    wait_for_picked(&ctx, "Doing", "Shop", true).await?;

    let wash_pin = hover_till_pin_shows(&ctx, "To do", "Wash").await?;
    wash_pin.click().await?;

    wait_for_pinned(&ctx, "Wash").await?;
    wait_for_groups(&ctx, "To do: Wash, Cook, Sweep; Doing: Shop; Done: nothing").await?;
    wait_for_picked(&ctx, "Doing", "Shop", true).await
}

async fn wait_for_opacity(ctx: &Ctx, label: &str, expected: &str, why: &str) -> Result<()> {
    let pin = ctx.find(pin(label)).await?;
    ctx.wait_for(why, async || {
        Ok((opacity(&pin).await? == expected).then_some(()))
    })
    .await
}

async fn press_tab_to(ctx: &Ctx, label: &str) -> Result<()> {
    ctx.action_chain().send_keys(Key::Tab).perform().await?;
    let focused = ctx.active_element().await?;
    let name = match focused.attr("aria-label").await? {
        Some(name) => name,
        None => focused.text().await?,
    };
    ensure!(name == label, "Tab went to {name:?}, not to {label:?}");
    Ok(())
}

/// The keyboard on the button of an item shows what stands at its end, and Tab goes on to it,
/// and further on, out of the item, hides it.
pub async fn aside_shows_with_the_keyboard_on_the_item(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    // From the pin of the chore before, so that the keyboard brings the focus to the chore
    ctx.find(pin("Wash")).await?.focus().await?;
    press_tab_to(&ctx, "Cook").await?;
    wait_for_opacity(&ctx, "Cook", "1", "the pin to show").await?;

    press_tab_to(&ctx, "Pin Cook").await?;
    ensure!(
        opacity(&ctx.find(pin("Cook")).await?).await? == "1",
        "the pin in focus stays in sight"
    );

    press_tab_to(&ctx, "Sweep").await?;
    wait_for_opacity(&ctx, "Cook", "0", "the pin to hide once the keyboard left").await
}

/// Leaving the item with the mouse hides what stands at its end - also after a click on the
/// item, which focuses its button but isn't the keyboard. While the item is picked, it shows.
pub async fn aside_hides_once_the_mouse_leaves(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    let away = ctx.find(By::XPath(SUMMARY.to_string())).await?;
    let leave = async || {
        ctx.action_chain()
            .move_to_element_center(&away)
            .perform()
            .await
    };

    hover_till_pin_shows(&ctx, "To do", "Cook").await?;
    leave().await?;
    wait_for_opacity(&ctx, "Cook", "0", "the pin to hide away from the chore").await?;

    let cook = ctx.find(item("To do", "Cook")).await?;
    cook.click().await?;
    wait_for_picked(&ctx, "To do", "Cook", true).await?;
    leave().await?;
    ensure!(
        opacity(&ctx.find(pin("Cook")).await?).await? == "1",
        "the picked chore shows its pin"
    );

    cook.click().await?;
    wait_for_picked(&ctx, "To do", "Cook", false).await?;
    leave().await?;
    wait_for_opacity(
        &ctx,
        "Cook",
        "0",
        "the pin to hide, though Cook has the focus",
    )
    .await
}

/// Taps `target` with a finger, `x` CSS pixels right of its center.
async fn tap(ctx: &Ctx, target: &WebElement, x: i64) -> Result<()> {
    let actions = json!([{
        "type": "pointer",
        "id": "finger",
        "parameters": { "pointerType": "touch" },
        "actions": [
            { "type": "pointerMove", "origin": target.to_json()?, "x": x, "y": 0, "duration": 0 },
            { "type": "pointerDown", "button": 0 },
            { "type": "pointerUp", "button": 0 },
        ],
    }]);
    ctx.cmd(Command::PerformActions(Actions::from(actions)))
        .await?;
    Ok(())
}

/// A finger can't see a hidden aside, so a tap where it stands picks the item. The picked item
/// shows it, and a tap on it does what it does.
pub async fn tap_on_hidden_aside_picks_the_item(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    let wash_pin = ctx.find(pin("Wash")).await?;

    tap(&ctx, &wash_pin, 0).await?;
    wait_for_picked(&ctx, "To do", "Wash", true).await?;
    wait_for_pinned(&ctx, "nothing").await?;
    wait_for_opacity(&ctx, "Wash", "1", "the picked chore to show its pin").await?;

    tap(&ctx, &wash_pin, 0).await?;
    wait_for_pinned(&ctx, "Wash").await?;
    wait_for_picked(&ctx, "To do", "Wash", true).await?;

    // On its left, away from the pin: a second tap puts the chore down
    let wash = ctx.find(item("To do", "Wash")).await?;
    let quarter = (wash.rect().await?.width / 4.0) as i64;
    tap(&ctx, &wash, -quarter).await?;
    wait_for_picked(&ctx, "To do", "Wash", false).await?;

    // A pinned chore shows its pin, which a tap reaches
    tap(&ctx, &wash_pin, 0).await?;
    wait_for_pinned(&ctx, "nothing").await?;
    wait_for_picked(&ctx, "To do", "Wash", false).await?;
    wait_for_opacity(&ctx, "Wash", "0", "the pin to hide after the taps").await
}

/// Enter on what stands at the end of an item, which moves the item - the pin puts it first in
/// its group: the focus stays with the item, on its button.
pub async fn focus_stays_with_the_item_its_aside_moves(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    // From the pin of the chore before, so that the keyboard brings the focus to the chore
    ctx.find(pin("Wash")).await?.focus().await?;
    press_tab_to(&ctx, "Cook").await?;
    press_tab_to(&ctx, "Pin Cook").await?;

    ctx.action_chain().send_keys(Key::Enter).perform().await?;

    wait_for_pinned(&ctx, "Cook").await?;
    wait_for_groups(&ctx, "To do: Cook, Wash, Sweep; Doing: Shop; Done: nothing").await?;
    wait_for_focus(&ctx, "To do/Cook").await
}

/// Escape on what stands at the end of an item is left to it, and to the page: the picked chore
/// stays picked. Escape on the chore puts it down.
pub async fn escape_on_the_aside_is_left_to_the_page(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    ctx.find(item("To do", "Wash")).await?.focus().await?;
    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_for_picked(&ctx, "To do", "Wash", true).await?;
    ctx.js("window.escapes = 0; \
            document.addEventListener('keydown', (e) => { if (e.key === 'Escape') window.escapes += 1; })")
        .await?;

    press_tab_to(&ctx, "Pin Wash").await?;
    ctx.action_chain().send_keys(Key::Escape).perform().await?;
    let escapes = ctx.js("return window.escapes").await?;
    ensure!(
        escapes.as_i64() == Some(1),
        "the page heard Escape {escapes} times"
    );
    wait_for_picked(&ctx, "To do", "Wash", true).await?;

    ctx.action_chain()
        .key_down(Key::Shift)
        .send_keys(Key::Tab)
        .key_up(Key::Shift)
        .perform()
        .await?;
    wait_for_focus(&ctx, "To do/Wash").await?;
    ctx.action_chain().send_keys(Key::Escape).perform().await?;
    wait_for_picked(&ctx, "To do", "Wash", false).await
}

/// A group which stretches its items, as a column does: the button of an item fills its box,
/// which the pin over its end takes no room of.
pub async fn stretched_item_fills_its_box(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    let widths = ctx
        .js(
            "const items = document.querySelector(\"[role='group'][aria-label='To do'] > div\"); \
             items.style.cssText = 'display: flex; flex-direction: column; align-items: stretch; \
                 width: 300px'; \
             const button = [...items.querySelectorAll('button')] \
                 .find((b) => b.textContent.trim() === 'Wash'); \
             return [button.getBoundingClientRect().width, \
                 button.parentElement.getBoundingClientRect().width]",
        )
        .await?;
    let width = |at: usize| widths.get(at).and_then(|width| width.as_f64());
    let (Some(button), Some(item_box)) = (width(0), width(1)) else {
        bail!("no widths in {widths}");
    };
    ensure!(
        (item_box - 300.0).abs() < 1.0,
        "the box is {item_box} wide, not stretched"
    );
    ensure!(
        (button - item_box).abs() < 1.0,
        "the button is {button} wide, its box {item_box}"
    );
    Ok(())
}

/// What stands below the items of a group stays as they change: the focus stays on "Add a chore",
/// pressed with the keyboard again.
pub async fn focus_below_the_items_stays_as_they_change(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    ctx.find(By::XPath(
        "//button[normalize-space(.)='Add a chore']".to_string(),
    ))
    .await?
    .focus()
    .await?;

    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_for_groups(
        &ctx,
        "To do: Wash, Cook, Sweep, Chore 1; Doing: Shop; Done: nothing",
    )
    .await?;
    wait_for_focus(&ctx, "To do/Add a chore").await?;

    ctx.action_chain().send_keys(Key::Enter).perform().await?;
    wait_for_groups(
        &ctx,
        "To do: Wash, Cook, Sweep, Chore 1, Chore 2; Doing: Shop; Done: nothing",
    )
    .await
}
