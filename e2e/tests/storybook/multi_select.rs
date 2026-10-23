//! Story "MultiSelect": `MultiSelect` of strings, a button for each option, and one as a filter
//! (`MultiSelectMode::Exclusive` with "All").

use vertigo_forms_e2e::prelude::*;

pub const SELECTED: &str = "//p[starts-with(normalize-space(.), 'Selected values:')]";
const FILTERED: &str = "//p[starts-with(normalize-space(.), 'Filtered by:')]";

/// Waits until exactly the `selected` options are marked as chosen (in bold).
pub async fn wait_for_marked(ctx: &Ctx, selected: &[&str]) -> Result<()> {
    ctx.wait_for(&format!("options {selected:?} to be marked"), async || {
        for option in ["foo", "bar", "baz"] {
            let button = ctx.button(option).await?;
            let bold = ctx.style(&button, "font-weight").await? == "700";
            if bold != selected.contains(&option) {
                return Ok(None);
            }
        }
        Ok(Some(()))
    })
    .await
}

pub async fn buttons_toggle_values(ctx: Ctx) -> Result<()> {
    let selected = By::XPath(SELECTED);

    ctx.open("/multi_select").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: foo")
        .await?;
    wait_for_marked(&ctx, &["foo"]).await?;

    ctx.click_button("bar").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: foo,bar")
        .await?;
    ctx.click_button("foo").await?;
    ctx.wait_for_text_of(selected.clone(), "Selected values: bar")
        .await?;
    ctx.click_button("baz").await?;
    ctx.wait_for_text_of(selected, "Selected values: bar,baz")
        .await?;
    wait_for_marked(&ctx, &["bar", "baz"]).await
}

/// Waits until exactly the `pressed` buttons of the filter say so in `aria-pressed`.
async fn wait_for_pressed(ctx: &Ctx, pressed: &[&str]) -> Result<()> {
    let filter_buttons = |state: &str| {
        By::XPath(format!(
            "//div[starts-with(normalize-space(.), 'Filter')]//button[@aria-pressed='{state}']"
        ))
    };
    ctx.wait_for(
        &format!("filter buttons {pressed:?} to be pressed"),
        async || {
            let on = ctx.texts(filter_buttons("true")).await?;
            let off = ctx.texts(filter_buttons("false")).await?;
            Ok((on == pressed && on.len() + off.len() == 4).then_some(()))
        },
    )
    .await
}

/// Clicks the button `label` with Ctrl held.
async fn ctrl_click(ctx: &Ctx, label: &str) -> Result<()> {
    let button = ctx.button(label).await?;
    ctx.action_chain()
        .key_down(Key::Control)
        .click_element(&button)
        .key_up(Key::Control)
        .perform()
        .await?;
    Ok(())
}

pub async fn filter_click_selects_one_ctrl_click_adds(ctx: Ctx) -> Result<()> {
    let filtered = By::XPath(FILTERED);

    ctx.open("/multi_select").await?;
    ctx.wait_for_text_of(filtered.clone(), "Filtered by: red")
        .await?;
    wait_for_pressed(&ctx, &["red"]).await?;

    ctx.click_button("green").await?;
    ctx.wait_for_text_of(filtered.clone(), "Filtered by: green")
        .await?;
    ctrl_click(&ctx, "blue").await?;
    ctx.wait_for_text_of(filtered.clone(), "Filtered by: green,blue")
        .await?;
    wait_for_pressed(&ctx, &["green", "blue"]).await?;
    ctrl_click(&ctx, "green").await?;
    ctx.wait_for_text_of(filtered.clone(), "Filtered by: blue")
        .await?;

    // A plain click leaves only the clicked one, "All" none
    ctrl_click(&ctx, "red").await?;
    ctx.click_button("red").await?;
    ctx.wait_for_text_of(filtered.clone(), "Filtered by: red")
        .await?;
    ctx.click_button("All").await?;
    ctx.wait_for_text_of(filtered, "Filtered by: all").await?;
    wait_for_pressed(&ctx, &["All"]).await
}
