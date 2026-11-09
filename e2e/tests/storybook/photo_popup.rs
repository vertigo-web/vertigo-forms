//! Story "Photo Popup": `PhotoGallery` of three photos, and one of a single photo with
//! custom labels.

use thirtyfour::common::command::{Actions, Command};
use vertigo_forms_e2e::prelude::*;

const DIALOG: &str = "//*[@role='dialog']";

fn dialog() -> By {
    By::XPath(DIALOG)
}

fn thumbnail(number: u32) -> By {
    By::XPath(format!("//a/img[@alt='Photo {number}']"))
}

fn photo() -> By {
    By::XPath(format!("{DIALOG}/img"))
}

fn counter() -> By {
    By::XPath(format!("{DIALOG}/span"))
}

fn dialog_button(title: &str) -> By {
    By::XPath(format!("{DIALOG}/button[@title={}]", xpath_literal(title)))
}

async fn open_story(ctx: &Ctx) -> Result<()> {
    ctx.open("/photo_popup").await?;
    ctx.find(thumbnail(1)).await?;
    ctx.wait_for_none(dialog()).await
}

async fn click_thumbnail(ctx: &Ctx, number: u32) -> Result<()> {
    ctx.find(thumbnail(number)).await?.click().await?;
    Ok(())
}

/// Waits until the popup shows photo `number`, the same picture as its thumbnail.
async fn wait_for_photo(ctx: &Ctx, number: u32) -> Result<()> {
    let alt = format!("Photo {number}");
    let url = ctx
        .find(thumbnail(number))
        .await?
        .attr("src")
        .await?
        .unwrap_or_default();
    ctx.wait_for(&format!("the popup to show {alt:?}"), async || {
        let shown = ctx.find_all(photo()).await?;
        let Some(shown) = shown.first() else {
            return Ok(None);
        };
        let matches = shown.attr("alt").await?.as_deref() == Some(alt.as_str())
            && shown.attr("src").await?.as_deref() == Some(url.as_str());
        Ok(matches.then_some(()))
    })
    .await
}

async fn press(ctx: &Ctx, key: Key) -> Result<()> {
    ctx.action_chain().send_keys(key).perform().await?;
    Ok(())
}

/// Touches `element` with a finger, `from` its center, and moves the finger `by` before lifting
/// it, as on a touch screen.
async fn swipe(ctx: &Ctx, element: &WebElement, from: (i64, i64), by: (i64, i64)) -> Result<()> {
    swipe_with(ctx, "touch", element, from, by).await
}

/// [`swipe`] with a pointer of `pointer_type`, `touch` or `pen`.
async fn swipe_with(
    ctx: &Ctx,
    pointer_type: &str,
    element: &WebElement,
    from: (i64, i64),
    by: (i64, i64),
) -> Result<()> {
    let actions = json!([{
        "type": "pointer",
        "id": pointer_type,
        "parameters": { "pointerType": pointer_type },
        "actions": [
            {
                "type": "pointerMove",
                "origin": element.to_json()?,
                "x": from.0,
                "y": from.1,
                "duration": 0,
            },
            { "type": "pointerDown", "button": 0 },
            { "type": "pointerMove", "origin": "pointer", "x": by.0, "y": by.1, "duration": 200 },
            { "type": "pointerUp", "button": 0 },
        ],
    }]);
    ctx.cmd(Command::PerformActions(Actions::from(actions)))
        .await?;
    Ok(())
}

/// Offset from the center of the popup to near its top left corner, where there's no photo nor
/// button.
async fn corner(ctx: &Ctx) -> Result<(i64, i64)> {
    let rect = ctx.find(dialog()).await?.rect().await?;
    Ok((
        (10.0 - rect.width / 2.0) as i64,
        (10.0 - rect.height / 2.0) as i64,
    ))
}

pub async fn thumbnail_opens_its_photo(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    ctx.mark_document().await?;

    click_thumbnail(&ctx, 2).await?;
    wait_for_photo(&ctx, 2).await?;
    ctx.wait_for_text_of(counter(), "2 / 3").await?;
    // In the app, not in a new tab or page
    ctx.assert_same_document().await?;
    ensure!(
        ctx.windows().await?.len() == 1,
        "the photo opened in a new tab as well"
    );
    // The keyboard is in the popup
    let focused = ctx
        .js("return document.activeElement.getAttribute('role')")
        .await?;
    ensure!(focused == json!("dialog"), "focus on {focused}");

    ctx.find(dialog_button("Close")).await?.click().await?;
    ctx.wait_for_none(dialog()).await
}

pub async fn buttons_and_arrow_keys_go_around(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;

    click_thumbnail(&ctx, 3).await?;
    ctx.wait_for_text_of(counter(), "3 / 3").await?;
    ctx.find(dialog_button("Next")).await?.click().await?;
    wait_for_photo(&ctx, 1).await?;
    ctx.wait_for_text_of(counter(), "1 / 3").await?;
    ctx.find(dialog_button("Previous")).await?.click().await?;
    wait_for_photo(&ctx, 3).await?;

    press(&ctx, Key::Left).await?;
    wait_for_photo(&ctx, 2).await?;
    ctx.wait_for_text_of(counter(), "2 / 3").await?;
    press(&ctx, Key::Right).await?;
    wait_for_photo(&ctx, 3).await?;
    press(&ctx, Key::Escape).await?;
    ctx.wait_for_none(dialog()).await
}

pub async fn click_beside_photo_closes(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    click_thumbnail(&ctx, 1).await?;
    wait_for_photo(&ctx, 1).await?;

    // A click on the photo leaves it open
    ctx.find(photo()).await?.click().await?;
    press(&ctx, Key::Right).await?;
    wait_for_photo(&ctx, 2).await?;

    let (x, y) = corner(&ctx).await?;
    let layer = ctx.find(dialog()).await?;
    ctx.action_chain()
        .move_to_element_with_offset(&layer, x, y)
        .click()
        .perform()
        .await?;
    ctx.wait_for_none(dialog()).await
}

pub async fn swipe_goes_through_photos(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    click_thumbnail(&ctx, 1).await?;
    wait_for_photo(&ctx, 1).await?;

    // To the left the next photo, to the right the previous one
    swipe(&ctx, &ctx.find(photo()).await?, (0, 0), (-150, 0)).await?;
    wait_for_photo(&ctx, 2).await?;
    ctx.wait_for_text_of(counter(), "2 / 3").await?;
    swipe(&ctx, &ctx.find(photo()).await?, (0, 0), (150, 20)).await?;
    wait_for_photo(&ctx, 1).await?;

    // Too short, and more down than across
    swipe(&ctx, &ctx.find(photo()).await?, (0, 0), (-20, 0)).await?;
    swipe(&ctx, &ctx.find(photo()).await?, (0, -50), (-80, 120)).await?;
    swipe(&ctx, &ctx.find(photo()).await?, (0, 0), (-150, 0)).await?;
    wait_for_photo(&ctx, 2).await?;

    // Beside the photo too, and it doesn't close the popup. A pen, unlike a finger, ends a swipe
    // with a click.
    let (x, y) = corner(&ctx).await?;
    swipe(&ctx, &ctx.find(dialog()).await?, (x, y), (150, 0)).await?;
    wait_for_photo(&ctx, 1).await?;
    swipe_with(&ctx, "pen", &ctx.find(dialog()).await?, (x, y), (150, 0)).await?;
    wait_for_photo(&ctx, 3).await?;
    press(&ctx, Key::Right).await?;
    wait_for_photo(&ctx, 1).await?;

    // A tap beside the photo still closes it
    swipe(&ctx, &ctx.find(dialog()).await?, (x, y), (0, 0)).await?;
    ctx.wait_for_none(dialog()).await
}

pub async fn single_photo_has_no_browsing(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    click_thumbnail(&ctx, 4).await?;
    wait_for_photo(&ctx, 4).await?;

    let mut titles = Vec::new();
    for button in ctx.find_all(By::XPath(format!("{DIALOG}/button"))).await? {
        titles.push(button.attr("title").await?.unwrap_or_default());
    }
    ensure!(titles == ["Close photo"], "buttons {titles:?}");
    ensure!(
        ctx.find_all(counter()).await?.is_empty(),
        "a single photo has a counter"
    );
    press(&ctx, Key::Right).await?;
    swipe(&ctx, &ctx.find(photo()).await?, (0, 0), (-150, 0)).await?;
    wait_for_photo(&ctx, 4).await?;

    ctx.find(dialog_button("Close photo"))
        .await?
        .click()
        .await?;
    ctx.wait_for_none(dialog()).await
}

/// Before the app starts, for search engines and with a modifier key, a thumbnail is a link to
/// its photo, opened in a new tab.
pub async fn thumbnails_link_to_photos(ctx: Ctx) -> Result<()> {
    open_story(&ctx).await?;
    for number in 1..=4 {
        let img = ctx.find(thumbnail(number)).await?;
        let src = img.attr("src").await?;
        let link = img.parent().await?;
        ensure!(
            link.attr("href").await? == src && src.is_some(),
            "photo {number}: link to {:?}",
            link.attr("href").await?
        );
        ensure!(
            link.attr("target").await?.as_deref() == Some("_blank"),
            "photo {number} opens in the same tab"
        );
    }
    // `t:` attributes go to the thumbnails
    let loading = ctx.find(thumbnail(1)).await?.attr("loading").await?;
    ensure!(loading.as_deref() == Some("lazy"), "loading {loading:?}");
    Ok(())
}
