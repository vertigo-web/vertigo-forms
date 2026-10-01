//! Story "Form": three `ModelForm`s in tabs. "Form 1" and "Form 2" show their model below the
//! form, so it's visible what was submitted. "Tabbed Form" has its fields in tabs.

use vertigo_forms_e2e::prelude::*;

const ORIGINAL_PHOTO: &str = "https://picsum.photos/200";

/// Field of the open form by its `name`.
fn field(name: &str) -> By {
    By::XPath(format!("//form//*[@name={}]", xpath_literal(name)))
}

/// `<select>` of the form section labelled `label`.
fn select(label: &str) -> By {
    By::XPath(format!(
        "//form//label[text()[normalize-space(.)={}]]//select",
        xpath_literal(label)
    ))
}

fn submit(label: &str) -> By {
    By::XPath(format!(
        "//form//input[@type='submit' and @value={}]",
        xpath_literal(label)
    ))
}

/// The `n`-th line of the model shown below the form.
fn model_line(heading: &str, n: usize) -> By {
    By::XPath(format!(
        "//h4[normalize-space(.)={}]/following-sibling::p[{n}]",
        xpath_literal(heading)
    ))
}

async fn fill(ctx: &Ctx, fields: &[(&str, &str)]) -> Result<()> {
    for (name, value) in fields {
        ctx.retype(&ctx.find(field(name)).await?, value).await?;
    }
    Ok(())
}

async fn selected(ctx: &Ctx, label: &str) -> Result<String> {
    let select = SelectElement::new(&ctx.find(select(label)).await?).await?;
    Ok(select.first_selected_option().await?.text().await?)
}

pub async fn form_1_submits_changes(ctx: Ctx) -> Result<()> {
    let fields = [
        ("slug", "model-two"),
        ("name", "Model Two"),
        ("dimension_x", "200"),
        ("dimension_y", "150"),
    ];

    ctx.open("/form").await?;
    ctx.wait_for_tab_active("Form 1", true).await?;
    ctx.wait_for_text_of(model_line("Model 1:", 1), "model-one / Model One")
        .await?;
    ctx.wait_for_text_of(model_line("Model 1:", 2), "120x80")
        .await?;
    // The dimensions are one section, with "x" between the fields
    let dimensions = ctx
        .text_of(By::XPath(
            "//form/label[text()[normalize-space(.)='Dimensions']]",
        ))
        .await?;
    ensure!(dimensions == "Dimensions x", "dimensions: {dimensions:?}");

    fill(&ctx, &fields).await?;
    ensure!(
        ctx.text_of(model_line("Model 1:", 1)).await? == "model-one / Model One",
        "the model changed before submitting"
    );

    ctx.find(submit("Submit")).await?.click().await?;
    ctx.wait_for_text_of(model_line("Model 1:", 1), "model-two / Model Two")
        .await?;
    ctx.wait_for_text_of(model_line("Model 1:", 2), "200x150")
        .await?;
    // The form is rendered anew from the submitted model
    for (name, value) in fields {
        ctx.wait_for_value(field(name), value).await?;
    }
    Ok(())
}

pub async fn form_1_submits_on_enter(ctx: Ctx) -> Result<()> {
    ctx.open("/form").await?;
    let name = ctx.find(field("name")).await?;
    ctx.retype(&name, "Entered").await?;
    name.send_keys(Key::Enter).await?;
    ctx.wait_for_text_of(model_line("Model 1:", 1), "model-one / Entered")
        .await
}

async fn open_form_2(ctx: &Ctx) -> Result<()> {
    ctx.allow_sample_pictures();
    ctx.open("/form").await?;
    ctx.click_tab("Form 2").await?;
    ctx.wait_for_text_of(model_line("Model 2:", 1), "Johann / Gambolputty (Male)")
        .await
}

/// `<img>` of the "Photo" section of the form.
fn photo_preview() -> By {
    By::XPath("//form//label[text()[normalize-space(.)='Photo']]//img")
}

/// The photo of the model, below the form.
fn model_photo() -> By {
    By::XPath("//h4[normalize-space(.)='Model 2:']/following-sibling::p[3]/img")
}

async fn wait_for_src(ctx: &Ctx, img: By, expected: &str) -> Result<()> {
    let mut last = String::new();
    ctx.wait_for(&format!("picture {expected:.60}"), async || {
        last = ctx
            .find(img.clone())
            .await?
            .attr("src")
            .await?
            .unwrap_or_default();
        Ok((last == expected).then_some(()))
    })
    .await
    .with_context(|| format!("last picture: {last:.60}"))
}

async fn choose_photo(ctx: &Ctx, name: &str) -> Result<String> {
    let file = ctx.picture_file(name)?;
    ctx.find(By::XPath(
        "//form//label[text()[normalize-space(.)='Photo']]//input[@type='file']",
    ))
    .await?
    .send_keys(file.to_string_lossy())
    .await?;
    ctx.wait_for_text(&format!("{name} (69)")).await?;

    let src = ctx
        .find(photo_preview())
        .await?
        .attr("src")
        .await?
        .unwrap_or_default();
    ensure!(
        src.starts_with("data:image/png;base64,"),
        "preview of the chosen photo: {src:.60}"
    );
    Ok(src)
}

pub async fn form_2_submits_selects(ctx: Ctx) -> Result<()> {
    open_form_2(&ctx).await?;
    ctx.wait_for_text_of(model_line("Model 2:", 2), "Role: 1")
        .await?;
    ensure!(selected(&ctx, "Gender").await? == "Male");
    ensure!(selected(&ctx, "Role").await? == "Admin");

    fill(&ctx, &[("first_name", "Jan"), ("surname", "Kowalski")]).await?;
    SelectElement::new(&ctx.find(select("Gender")).await?)
        .await?
        .select_by_exact_text("Female")
        .await?;
    SelectElement::new(&ctx.find(select("Role")).await?)
        .await?
        .select_by_exact_text("Reporter")
        .await?;

    ctx.find(submit("Apply")).await?.click().await?;
    ctx.wait_for_text_of(model_line("Model 2:", 1), "Jan / Kowalski (Female)")
        .await?;
    ctx.wait_for_text_of(model_line("Model 2:", 2), "Role: 3")
        .await?;
    wait_for_src(&ctx, model_photo(), ORIGINAL_PHOTO).await?;
    ensure!(selected(&ctx, "Gender").await? == "Female");
    ensure!(selected(&ctx, "Role").await? == "Reporter");
    Ok(())
}

pub async fn form_2_submits_chosen_photo(ctx: Ctx) -> Result<()> {
    open_form_2(&ctx).await?;
    let photo = choose_photo(&ctx, "photo.png").await?;
    wait_for_src(&ctx, model_photo(), ORIGINAL_PHOTO).await?;

    ctx.find(submit("Apply")).await?.click().await?;
    wait_for_src(&ctx, model_photo(), &photo).await
}

/// "Revert", a button inside the `<form>`, restores the photo without submitting the form.
pub async fn form_2_reverts_photo_without_submitting(ctx: Ctx) -> Result<()> {
    open_form_2(&ctx).await?;
    fill(&ctx, &[("first_name", "Unsaved")]).await?;
    choose_photo(&ctx, "photo.png").await?;

    ctx.click_button("Revert").await?;
    wait_for_src(&ctx, photo_preview(), ORIGINAL_PHOTO).await?;
    ctx.wait_for_no_text("photo.png (69)").await?;
    ctx.wait_for_none(By::XPath("//button[normalize-space(.)='Revert']"))
        .await?;

    ensure!(
        ctx.text_of(model_line("Model 2:", 1)).await? == "Johann / Gambolputty (Male)",
        "the form was submitted"
    );
    ctx.wait_for_value(field("first_name"), "Unsaved").await
}

async fn open_tabbed_form(ctx: &Ctx) -> Result<()> {
    ctx.open("/form").await?;
    ctx.click_tab("Tabbed Form").await?;
    ctx.by_text("h4", "Tabbed Form:").await?;
    ctx.wait_for_tab_active("Basic", true).await
}

pub async fn tabbed_form_keeps_values_across_tabs(ctx: Ctx) -> Result<()> {
    open_tabbed_form(&ctx).await?;
    ctx.wait_for_value(field("first_name"), "Johann").await?;
    fill(&ctx, &[("last_name", "Smith")]).await?;

    ctx.click_tab("Other").await?;
    ctx.wait_for_tab_active("Other", true).await?;
    ctx.wait_for_value(field("annotation"), "").await?;
    ctx.find(field("annotation"))
        .await?
        .send_keys("A note")
        .await?;

    ctx.click_tab("Basic").await?;
    ctx.wait_for_value(field("last_name"), "Smith").await?;
    ctx.click_tab("Other").await?;
    ctx.wait_for_value(field("annotation"), "A note").await
}

/// The submit button above the tabs sends the fields of all of them.
pub async fn tabbed_form_submits_fields_of_all_tabs(ctx: Ctx) -> Result<()> {
    open_tabbed_form(&ctx).await?;
    fill(&ctx, &[("first_name", "Hans")]).await?;
    ctx.click_tab("Other").await?;
    ctx.find(field("annotation"))
        .await?
        .send_keys("A note")
        .await?;

    let form = ctx.find(By::Tag("form")).await?;
    ctx.find(submit("Submit")).await?.click().await?;
    // The story doesn't show the model, but the form is rendered anew from it
    ctx.wait_for("the form to be rendered anew", async || {
        Ok((!form.is_present().await?).then_some(()))
    })
    .await?;

    ctx.click_tab("Basic").await?;
    ctx.wait_for_value(field("first_name"), "Hans").await?;
    ctx.wait_for_value(field("last_name"), "Gambolputty")
        .await?;
    ctx.click_tab("Other").await?;
    ctx.wait_for_value(field("annotation"), "A note").await
}
