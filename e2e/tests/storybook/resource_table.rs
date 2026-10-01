//! Story "Resource Table": two `ResourceTable`s of the same items, one with a form made by
//! hand (a text field), the other with a form from a `DataSection` (`row_from_data_section`).
//! Their callbacks change the story's list at once, and never fail.

use vertigo_forms_e2e::prelude::*;

const TEXT: &str = "My Resources";
const DATA: &str = "My Resources (DataSection)";
const ADD: &str = "Add Item";
const QUESTION: &str = "Are you sure?";
const EMPTY_NAME: &str = "Name cannot be empty";
const SAVE_ERROR: &str = "Server error: the changes weren't saved";
const ADD_ERROR: &str = "Server error: the item wasn't added";
const DELETE_ERROR: &str = "Server error: the item wasn't deleted";

const TEXT_ROWS: &[&[&str]] = &[&["1", "Item 1"], &["2", "Item 2"]];
const DATA_ROWS: &[&[&str]] = &[
    &["1", "Item 1", "Yes", "Admin"],
    &["2", "Item 2", "No", "User"],
];

async fn open(ctx: &Ctx) -> Result<()> {
    ctx.open("/resource_table").await?;
    ctx.table(TEXT).wait_for_rows(TEXT_ROWS).await
}

/// Makes the story's server reject every change, or accept them again.
async fn server_rejects(ctx: &Ctx, rejects: bool) -> Result<()> {
    let switch = By::XPath("//label[normalize-space(.)='Server rejects changes']/input");
    if ctx.find(switch.clone()).await?.is_selected().await? != rejects {
        ctx.find(switch.clone()).await?.click().await?;
    }
    // `Switch` renders the checkbox anew on every change
    ctx.wait_for(
        &format!("the server to reject changes: {rejects}"),
        async || {
            let checked = ctx.find(switch.clone()).await?.is_selected().await?;
            Ok((checked == rejects).then_some(()))
        },
    )
    .await
}

/// The name field of a form of either table.
async fn name_field(form: &WebElement) -> Result<WebElement> {
    Ok(form
        .find(By::XPath(".//input[not(@type='checkbox')]"))
        .await?)
}

async fn name(ctx: &Ctx, form: &WebElement) -> Result<String> {
    ctx.value(&name_field(form).await?).await
}

async fn rename(ctx: &Ctx, form: &WebElement, name: &str) -> Result<()> {
    ctx.retype(&name_field(form).await?, name).await
}

/// "Active" checkbox of a `DataSection` form. `Switch` renders it anew on every change.
async fn active(form: &WebElement) -> Result<WebElement> {
    Ok(form.find(By::Css("input[name='is_active']")).await?)
}

async fn wait_for_active(ctx: &Ctx, form: &WebElement, expected: bool) -> Result<()> {
    ctx.wait_for(&format!("checkbox to be {expected}"), async || {
        Ok((active(form).await?.is_selected().await? == expected).then_some(()))
    })
    .await
}

/// "Role" select of a `DataSection` form.
async fn role(form: &WebElement) -> Result<SelectElement> {
    Ok(SelectElement::new(&form.find(By::Tag("select")).await?).await?)
}

async fn selected_role(form: &WebElement) -> Result<String> {
    Ok(role(form)
        .await?
        .first_selected_option()
        .await?
        .text()
        .await?)
}

pub async fn tables_show_their_items(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let text = ctx.table(TEXT);
    let data = ctx.table(DATA);

    let header = text.header().await?;
    ensure!(header == ["ID", "Name", "Actions"], "header: {header:?}");
    let header = data.header().await?;
    ensure!(
        header == ["ID", "Name", "Active", "Role", "Actions"],
        "header: {header:?}"
    );
    data.wait_for_rows(DATA_ROWS).await?;
    ensure!(
        text.forms().await?.is_empty() && data.forms().await?.is_empty(),
        "a form is open from the start"
    );
    Ok(())
}

pub async fn edit_saves_item(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 1", "Edit").await?;
    let form = table.form().await?;
    ensure!(name(&ctx, &form).await? == "Item 1");
    rename(&ctx, &form, "Renamed").await?;
    table.click_form_button("Save").await?;
    table
        .wait_for_rows(&[&["1", "Renamed"], &["2", "Item 2"]])
        .await?;
    table.wait_for_no_form().await?;

    // The next edit starts from the saved item
    table.click_in_row("Renamed", "Edit").await?;
    let form = table.form().await?;
    ensure!(name(&ctx, &form).await? == "Renamed");

    // The other table's items are its own
    ctx.table(DATA).wait_for_rows(DATA_ROWS).await
}

/// "Cancel" drops what was typed in: the next edit starts from the item.
pub async fn cancel_drops_changes(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 1", "Edit").await?;
    rename(&ctx, &table.form().await?, "Changed").await?;
    table.click_form_button("Cancel").await?;
    table.wait_for_no_form().await?;
    table.wait_for_rows(TEXT_ROWS).await?;

    table.click_in_row("Item 1", "Edit").await?;
    let form = table.form().await?;
    let value = name(&ctx, &form).await?;
    ensure!(
        value == "Item 1",
        "the form kept the dropped change: {value:?}"
    );
    Ok(())
}

pub async fn empty_name_is_rejected(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 2", "Edit").await?;
    let form = table.form().await?;
    rename(&ctx, &form, "").await?;
    table.click_form_button("Save").await?;
    table.wait_for_text(EMPTY_NAME).await?;
    // Still editing
    table.wait_for_rows(&[&["1", "Item 1"]]).await?;
    ensure!(name(&ctx, &form).await?.is_empty());

    rename(&ctx, &form, "Fixed").await?;
    table.click_form_button("Save").await?;
    table
        .wait_for_rows(&[&["1", "Item 1"], &["2", "Fixed"]])
        .await?;
    table.wait_for_no_text(EMPTY_NAME).await
}

pub async fn add_appends_items(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(TEXT);

    for (item, id) in [("Item 3", "12"), ("Item 4", "13")] {
        let form = table.open_add_form(ADD).await?;
        ensure!(
            name(&ctx, &form).await?.is_empty(),
            "the new item has a name"
        );
        rename(&ctx, &form, item).await?;
        click_in_form(&form, ADD).await?;
        table.wait_for_no_form().await?;
        // The story's list gives the item its id
        table.wait_for_rows_ending(&[id, item]).await?;
    }
    table
        .wait_for_rows(&[
            &["1", "Item 1"],
            &["2", "Item 2"],
            &["12", "Item 3"],
            &["13", "Item 4"],
        ])
        .await
}

pub async fn add_rejects_empty_name_and_cancel_closes(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(TEXT);

    let form = table.open_add_form(ADD).await?;
    click_in_form(&form, ADD).await?;
    table.wait_for_text(EMPTY_NAME).await?;
    table.form().await?;

    click_in_form(&form, "Cancel").await?;
    table.wait_for_no_form().await?;
    table.wait_for_no_text(EMPTY_NAME).await?;
    table.wait_for_rows(TEXT_ROWS).await
}

pub async fn delete_asks_first(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 2", "Delete").await?;
    table.wait_for_text(QUESTION).await?;
    table.answer_delete(QUESTION, "Cancel").await?;
    table.wait_for_no_text(QUESTION).await?;
    table.wait_for_rows(TEXT_ROWS).await?;

    table.click_in_row("Item 2", "Delete").await?;
    table.answer_delete(QUESTION, "Confirm").await?;
    table.wait_for_rows(&[&["1", "Item 1"]]).await?;
    table.wait_for_no_text(QUESTION).await
}

/// Each row has its own form: saving one leaves the other open, with what was typed in.
pub async fn rows_keep_their_own_forms(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 1", "Edit").await?;
    table.click_in_row("Item 2", "Edit").await?;
    let forms = table.forms().await?;
    ensure!(forms.len() == 2, "{} forms open", forms.len());
    rename(&ctx, &forms[0], "One").await?;
    rename(&ctx, &forms[1], "Two").await?;

    click_in_form(&forms[1], "Save").await?;
    table.wait_for_rows(&[&["2", "Two"]]).await?;
    let forms = table.forms().await?;
    ensure!(forms.len() == 1, "{} forms open", forms.len());
    ensure!(name(&ctx, &forms[0]).await? == "One");

    click_in_form(&forms[0], "Save").await?;
    table.wait_for_rows(&[&["1", "One"], &["2", "Two"]]).await
}

pub async fn data_section_form_edits_all_fields(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(DATA);

    table.click_in_row("Item 2", "Edit").await?;
    let form = table.form().await?;
    let id = form.find(By::XPath("./div[1]")).await?.text().await?;
    ensure!(id == "2", "first cell of the form: {id:?}");
    ensure!(name(&ctx, &form).await? == "Item 2");
    wait_for_active(&ctx, &form, false).await?;
    ensure!(selected_role(&form).await? == "User");

    rename(&ctx, &form, "Second").await?;
    active(&form).await?.click().await?;
    wait_for_active(&ctx, &form, true).await?;
    role(&form).await?.select_by_exact_text("Admin").await?;
    table.click_form_button("Save").await?;
    table
        .wait_for_rows(&[
            &["1", "Item 1", "Yes", "Admin"],
            &["2", "Second", "Yes", "Admin"],
        ])
        .await?;

    table.click_in_row("Second", "Edit").await?;
    let form = table.form().await?;
    wait_for_active(&ctx, &form, true).await?;
    ensure!(selected_role(&form).await? == "Admin");
    Ok(())
}

pub async fn data_section_cancel_drops_changes(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(DATA);

    table.click_in_row("Item 1", "Edit").await?;
    let form = table.form().await?;
    rename(&ctx, &form, "Changed").await?;
    active(&form).await?.click().await?;
    wait_for_active(&ctx, &form, false).await?;
    role(&form).await?.select_by_exact_text("User").await?;
    table.click_form_button("Cancel").await?;
    table.wait_for_no_form().await?;
    table.wait_for_rows(DATA_ROWS).await?;

    table.click_in_row("Item 1", "Edit").await?;
    let form = table.form().await?;
    ensure!(name(&ctx, &form).await? == "Item 1");
    wait_for_active(&ctx, &form, true).await?;
    ensure!(selected_role(&form).await? == "Admin");
    Ok(())
}

pub async fn data_section_add_appends_items(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(DATA);

    let form = table.open_add_form(ADD).await?;
    let id = form.find(By::XPath("./div[1]")).await?.text().await?;
    ensure!(id == "0", "first cell of the form: {id:?}");
    ensure!(name(&ctx, &form).await?.is_empty());
    wait_for_active(&ctx, &form, false).await?;
    // No role yet, so an empty option is chosen
    ensure!(selected_role(&form).await?.is_empty());
    rename(&ctx, &form, "Third").await?;
    click_in_form(&form, ADD).await?;
    table.wait_for_no_form().await?;
    table
        .wait_for_rows_ending(&["12", "Third", "No", "Unknown"])
        .await?;

    let form = table.open_add_form(ADD).await?;
    rename(&ctx, &form, "Fourth").await?;
    active(&form).await?.click().await?;
    wait_for_active(&ctx, &form, true).await?;
    role(&form).await?.select_by_exact_text("User").await?;
    click_in_form(&form, ADD).await?;
    table
        .wait_for_rows(&[
            &["1", "Item 1", "Yes", "Admin"],
            &["2", "Item 2", "No", "User"],
            &["12", "Third", "No", "Unknown"],
            &["13", "Fourth", "Yes", "User"],
        ])
        .await
}

pub async fn data_section_rejects_empty_name(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(DATA);

    table.click_in_row("Item 1", "Edit").await?;
    let form = table.form().await?;
    rename(&ctx, &form, "").await?;
    table.click_form_button("Save").await?;
    table.wait_for_text(EMPTY_NAME).await?;
    table
        .wait_for_rows(&[&["2", "Item 2", "No", "User"]])
        .await?;

    table.click_form_button("Cancel").await?;
    table.wait_for_rows(DATA_ROWS).await?;
    table.wait_for_no_text(EMPTY_NAME).await
}

pub async fn data_section_delete(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    let table = ctx.table(DATA);

    table.click_in_row("Item 1", "Delete").await?;
    table.answer_delete(QUESTION, "Confirm").await?;
    table
        .wait_for_rows(&[&["2", "Item 2", "No", "User"]])
        .await?;
    ctx.table(TEXT).wait_for_rows(TEXT_ROWS).await
}

/// The table shows a saved change at once and gets the item back when the server rejects the
/// change. The row is rendered anew both times, and still returns to its form, with what was
/// typed in and the error.
pub async fn rejected_save_returns_to_form(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    server_rejects(&ctx, true).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 1", "Edit").await?;
    rename(&ctx, &table.form().await?, "Renamed").await?;
    table.click_form_button("Save").await?;
    // Until the server answers, the row shows the change, without buttons
    table.wait_for_text("Renamed").await?;
    let rows = table.rows().await?;
    ensure!(rows == [["2", "Item 2"]], "rows while saving: {rows:?}");

    table.wait_for_text(SAVE_ERROR).await?;
    let form = table.form().await?;
    let value = name(&ctx, &form).await?;
    ensure!(
        value == "Renamed",
        "the form lost what was typed in: {value:?}"
    );
    table.wait_for_rows(&[&["2", "Item 2"]]).await?;

    // Once the server accepts it, the same form saves
    server_rejects(&ctx, false).await?;
    click_in_form(&form, "Save").await?;
    table
        .wait_for_rows(&[&["1", "Renamed"], &["2", "Item 2"]])
        .await?;
    table.wait_for_no_text(SAVE_ERROR).await
}

/// "Cancel" after a rejected save shows the item as the server has it, and drops the error.
pub async fn cancel_after_rejected_save(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    server_rejects(&ctx, true).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 1", "Edit").await?;
    rename(&ctx, &table.form().await?, "Renamed").await?;
    table.click_form_button("Save").await?;
    table.wait_for_text(SAVE_ERROR).await?;

    table.click_form_button("Cancel").await?;
    table.wait_for_no_form().await?;
    table.wait_for_rows(TEXT_ROWS).await?;
    table.wait_for_no_text(SAVE_ERROR).await?;

    table.click_in_row("Item 1", "Edit").await?;
    let value = name(&ctx, &table.form().await?).await?;
    ensure!(
        value == "Item 1",
        "the form kept the rejected change: {value:?}"
    );
    Ok(())
}

/// The table drops a deleted row at once and gets it back when the server rejects the deletion,
/// with the error below it.
pub async fn rejected_delete_brings_row_back(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    server_rejects(&ctx, true).await?;
    let table = ctx.table(TEXT);

    table.click_in_row("Item 2", "Delete").await?;
    table.answer_delete(QUESTION, "Confirm").await?;
    table.wait_for_no_text("Item 2").await?;

    table.wait_for_text(DELETE_ERROR).await?;
    table.wait_for_rows(TEXT_ROWS).await?;

    // The next action on the row drops the error
    server_rejects(&ctx, false).await?;
    table.click_in_row("Item 2", "Delete").await?;
    table.wait_for_no_text(DELETE_ERROR).await?;
    table.answer_delete(QUESTION, "Confirm").await?;
    table.wait_for_rows(&[&["1", "Item 1"]]).await
}

pub async fn rejected_add_returns_to_form(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    server_rejects(&ctx, true).await?;
    let table = ctx.table(TEXT);

    let form = table.open_add_form(ADD).await?;
    rename(&ctx, &form, "Item 3").await?;
    click_in_form(&form, ADD).await?;
    table.wait_for_text(ADD_ERROR).await?;
    let form = table.form().await?;
    let value = name(&ctx, &form).await?;
    ensure!(
        value == "Item 3",
        "the form lost what was typed in: {value:?}"
    );
    table.wait_for_rows(TEXT_ROWS).await?;

    server_rejects(&ctx, false).await?;
    click_in_form(&form, ADD).await?;
    table.wait_for_no_form().await?;
    table.wait_for_rows_ending(&["12", "Item 3"]).await?;
    table.wait_for_no_text(ADD_ERROR).await
}

/// A rejected save keeps every field of a `DataSection` form as it was set.
pub async fn data_section_rejected_save_keeps_fields(ctx: Ctx) -> Result<()> {
    open(&ctx).await?;
    server_rejects(&ctx, true).await?;
    let table = ctx.table(DATA);

    table.click_in_row("Item 2", "Edit").await?;
    let form = table.form().await?;
    rename(&ctx, &form, "Second").await?;
    active(&form).await?.click().await?;
    wait_for_active(&ctx, &form, true).await?;
    role(&form).await?.select_by_exact_text("Admin").await?;
    table.click_form_button("Save").await?;

    table.wait_for_text(SAVE_ERROR).await?;
    let form = table.form().await?;
    ensure!(name(&ctx, &form).await? == "Second");
    wait_for_active(&ctx, &form, true).await?;
    ensure!(selected_role(&form).await? == "Admin");

    server_rejects(&ctx, false).await?;
    click_in_form(&form, "Save").await?;
    table
        .wait_for_rows(&[
            &["1", "Item 1", "Yes", "Admin"],
            &["2", "Second", "Yes", "Admin"],
        ])
        .await
}
