//! Story "Drop Image File": `DropImageFile` with an original picture, replaced by a picture
//! from the file chooser or dropped on it. The story shows the new picture below, too.

use vertigo_forms_e2e::prelude::*;

const ORIGINAL: &str = "https://picsum.photos/200";
/// Picture shown by the component.
const PREVIEW: &str = "//p[normalize-space(.)='Dropped image:']/preceding-sibling::div[1]//img";
/// Picture shown by the story, below.
const DROPPED: &str = "//p[normalize-space(.)='Dropped image:']/following-sibling::p[1]/img";
const CHOOSER: &str = "//label[normalize-space(.)='Select file...']/input[@type='file']";

async fn src(ctx: &Ctx, img: &str) -> Result<String> {
    Ok(ctx
        .find(By::XPath(img))
        .await?
        .attr("src")
        .await?
        .unwrap_or_default())
}

async fn wait_for_preview(ctx: &Ctx, expected: &str) -> Result<()> {
    ctx.wait_for(&format!("preview {expected:?}"), async || {
        Ok((src(ctx, PREVIEW).await? == expected).then_some(()))
    })
    .await
}

/// Waits until the component and the story show the new picture, named `name`.
async fn wait_for_new_picture(ctx: &Ctx, name: &str) -> Result<()> {
    ctx.wait_for_text(&format!("{name} (69)")).await?;
    let preview = src(ctx, PREVIEW).await?;
    ensure!(
        preview.starts_with("data:image/png;base64,"),
        "preview: {preview:.60}"
    );
    ensure!(
        src(ctx, DROPPED).await? == preview,
        "the story shows another picture"
    );

    // The browser can decode it
    let img = ctx.find(By::XPath(PREVIEW)).await?;
    ctx.wait_for("the preview to load", async || {
        let width = ctx
            .js_with(
                "return arguments[0].complete ? arguments[0].naturalWidth : null",
                vec![img.to_json()?],
            )
            .await?;
        Ok((width == json!(1)).then_some(()))
    })
    .await
}

pub async fn shows_original_picture(ctx: Ctx) -> Result<()> {
    ctx.allow_sample_pictures();
    ctx.open("/drop_file").await?;

    wait_for_preview(&ctx, ORIGINAL).await?;
    ctx.find(By::XPath(CHOOSER)).await?;
    ensure!(
        ctx.find_all(By::Tag("button")).await?.is_empty(),
        "nothing to revert, but there is a button"
    );
    ensure!(
        ctx.find_all(By::XPath(DROPPED)).await?.is_empty(),
        "the story shows a new picture"
    );
    Ok(())
}

pub async fn chosen_file_replaces_picture_until_reverted(ctx: Ctx) -> Result<()> {
    ctx.allow_sample_pictures();
    ctx.open("/drop_file").await?;

    let file = ctx.picture_file("chosen.png")?;
    ctx.find(By::XPath(CHOOSER))
        .await?
        .send_keys(file.to_string_lossy())
        .await?;
    wait_for_new_picture(&ctx, "chosen.png").await?;

    ctx.click_button("Revert").await?;
    wait_for_preview(&ctx, ORIGINAL).await?;
    ctx.wait_for_none(By::XPath(DROPPED)).await?;
    ctx.wait_for_no_text("chosen.png (69)").await
}

pub async fn dropped_file_replaces_picture(ctx: Ctx) -> Result<()> {
    ctx.allow_sample_pictures();
    ctx.open("/drop_file").await?;

    let preview = ctx.find(By::XPath(PREVIEW)).await?;
    ctx.drop_picture(&preview, "dropped.png").await?;
    wait_for_new_picture(&ctx, "dropped.png").await?;

    // Another picture replaces the dropped one
    let preview = ctx.find(By::XPath(PREVIEW)).await?;
    ctx.drop_picture(&preview, "again.png").await?;
    wait_for_new_picture(&ctx, "again.png").await?;
    ctx.wait_for_no_text("dropped.png (69)").await
}
