//! Story "Modal": a short window of the cat with a field and a button, and a wide one with
//! a long list of cats.

use vertigo_forms_e2e::prelude::*;

const DIALOG: &str = "//*[@role='dialog']";

fn dialog() -> By {
    By::XPath(DIALOG)
}

/// The layer over the page, under the window.
fn layer() -> By {
    By::XPath(format!("{DIALOG}/.."))
}

fn close_button(title: &str) -> By {
    By::XPath(format!("{DIALOG}/button[@title={}]", xpath_literal(title)))
}

fn name_field() -> By {
    By::XPath(format!("{DIALOG}//input[@name='name']"))
}

async fn open_cat(ctx: &Ctx) -> Result<()> {
    ctx.open("/modal").await?;
    ctx.wait_for_none(dialog()).await?;
    ctx.click_button("Show the cat").await?;
    ctx.find(dialog()).await?;
    Ok(())
}

async fn press(ctx: &Ctx, key: Key) -> Result<()> {
    ctx.action_chain().send_keys(key).perform().await?;
    Ok(())
}

async fn press_shift_tab(ctx: &Ctx) -> Result<()> {
    ctx.action_chain()
        .key_down(Key::Shift)
        .send_keys(Key::Tab)
        .key_up(Key::Shift)
        .perform()
        .await?;
    Ok(())
}

/// What has the focus: the role, name, title or text of the element.
async fn focused(ctx: &Ctx) -> Result<String> {
    let found = ctx
        .js("const e = document.activeElement; \
             return e.getAttribute('role') || e.getAttribute('name') || e.getAttribute('title') \
             || e.textContent.trim()")
        .await?;
    Ok(found.as_str().unwrap_or_default().to_string())
}

async fn wait_for_focus(ctx: &Ctx, expected: &str) -> Result<()> {
    ctx.wait_for(&format!("the focus on {expected:?}"), async || {
        Ok((focused(ctx).await? == expected).then_some(()))
    })
    .await
}

/// The `overflow` the page has in its own style, which the window sets while it is open.
async fn page_overflow(ctx: &Ctx) -> Result<String> {
    let overflow = ctx
        .js("return document.documentElement.style.overflow")
        .await?;
    Ok(overflow.as_str().unwrap_or_default().to_string())
}

/// The button opens the window, named by its title, with the keyboard in it and the page under
/// it kept from scrolling; Escape closes it.
pub async fn button_opens_window_named_by_its_title(ctx: Ctx) -> Result<()> {
    open_cat(&ctx).await?;

    let window = ctx.find(dialog()).await?;
    ensure!(
        window.attr("aria-modal").await?.as_deref() == Some("true"),
        "not modal"
    );
    let title = window.attr("aria-labelledby").await?.unwrap_or_default();
    let named = ctx.find(By::Id(&title)).await?.text().await?;
    ensure!(named == "The cat", "named {named:?}");
    wait_for_focus(&ctx, "dialog").await?;
    ensure!(page_overflow(&ctx).await? == "hidden", "the page scrolls");

    press(&ctx, Key::Escape).await?;
    ctx.wait_for_none(dialog()).await?;
    ensure!(
        page_overflow(&ctx).await?.is_empty(),
        "the page still locked"
    );
    Ok(())
}

/// A click in the window leaves it open, a click beside it and the close button close it.
pub async fn click_beside_or_close_button_closes(ctx: Ctx) -> Result<()> {
    open_cat(&ctx).await?;

    ctx.find(By::XPath(format!("{DIALOG}//p")))
        .await?
        .click()
        .await?;
    ctx.find(dialog()).await?;
    let layer = ctx.find(layer()).await?;
    let rect = layer.rect().await?;
    ctx.action_chain()
        .move_to_element_with_offset(
            &layer,
            (10.0 - rect.width / 2.0) as i64,
            (10.0 - rect.height / 2.0) as i64,
        )
        .click()
        .perform()
        .await?;
    ctx.wait_for_none(dialog()).await?;

    ctx.click_button("Show the cat").await?;
    ctx.find(close_button("Close")).await?.click().await?;
    ctx.wait_for_none(dialog()).await
}

/// Tab goes through the field, the button and the close button and comes back round to the
/// first, and Shift+Tab from the first goes to the close button: the keyboard doesn't leave the
/// window for the page under it.
pub async fn tab_stays_in_the_window(ctx: Ctx) -> Result<()> {
    open_cat(&ctx).await?;
    wait_for_focus(&ctx, "dialog").await?;

    for expected in ["name", "Pet", "Close", "dialog", "name"] {
        press(&ctx, Key::Tab).await?;
        // Past the close button the focus stops on the window itself, then goes on to the field
        if expected == "dialog" {
            wait_for_focus(&ctx, "dialog").await?;
            continue;
        }
        wait_for_focus(&ctx, expected).await?;
    }
    press_shift_tab(&ctx).await?;
    wait_for_focus(&ctx, "Close").await?;
    press_shift_tab(&ctx).await?;
    wait_for_focus(&ctx, "Pet").await
}

/// The content is rendered anew each time: a name typed in is gone when the window opens again.
pub async fn content_is_new_each_time(ctx: Ctx) -> Result<()> {
    open_cat(&ctx).await?;
    ctx.find(name_field()).await?.send_keys("Tom").await?;
    press(&ctx, Key::Escape).await?;
    ctx.wait_for_none(dialog()).await?;

    ctx.click_button("Show the cat").await?;
    let name = ctx.find(name_field()).await?.prop("value").await?;
    ensure!(name.as_deref() == Some(""), "the name is still {name:?}");
    Ok(())
}

/// A long content scrolls in the body of the window, which stays within the page, with the
/// title and the close button in sight; attributes of `w` go to the window.
pub async fn long_content_scrolls_in_the_window(ctx: Ctx) -> Result<()> {
    ctx.open("/modal").await?;
    ctx.click_button("Show all the cats").await?;
    let window = ctx.find(By::Id("cats")).await?;
    ensure!(
        window.attr("role").await?.as_deref() == Some("dialog"),
        "the id isn't on the window"
    );

    let sizes = ctx
        .js("const w = document.getElementById('cats'); \
             const body = w.children[1]; \
             return [w.getBoundingClientRect().bottom <= window.innerHeight, \
                     body.scrollHeight > body.clientHeight]")
        .await?;
    ensure!(sizes == json!([true, true]), "fits and scrolls: {sizes}");
    ctx.js("document.getElementById('cats').children[1].scrollTop = 100000")
        .await?;
    let close = ctx.find(close_button("Close the list")).await?;
    ensure!(
        close.is_displayed().await?,
        "the close button scrolled away"
    );
    let rect = close.rect().await?;
    ensure!(rect.y >= 0.0, "the close button is above the page");
    close.click().await?;
    ctx.wait_for_none(dialog()).await
}
