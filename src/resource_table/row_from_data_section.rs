use vertigo::{DomNode, css, dom};

use crate::{
    form::{DataSection, Field},
    resource_table::{base_row_css, normal_col_css},
};

/// Renders a single row in a `ResourceTable` using a `DataSection`.
///
/// This function simplifies building editable table rows by leveraging `vertigo_forms::form::DataSection`
/// to automatically render form fields based on the section's definition.
///
/// # Arguments
///
/// * `section` - A reference to the `DataSection` containing the row's form fields.
/// * `buttons` - A `DomNode` containing the action buttons (e.g., Save, Cancel) to render in the last column.
/// * `grid_template_columns` - A string specifying the CSS `grid-template-columns` property to align the
///   row's cells with the table header (e.g., `"50px 1fr 100px 150px"`).
///
/// # Example
///
/// ```rust,ignore
/// render_row_form: |form: &Rc<DataSection>, buttons| {
///     row_from_data_section(form, buttons, "50px 1fr 100px 150px")
/// }
/// ```
pub fn row_from_data_section(
    section: &DataSection,
    buttons: DomNode,
    grid_template_columns: &str,
) -> DomNode {
    let fields: Vec<DomNode> = section
        .fields
        .iter()
        .map(|field| {
            dom! { <Field {field} /> }
        })
        .collect();

    let error_node = section.error.as_ref().map(|err| {
        dom! { <span css={css! {"color: red; font-size: 0.8rem;"}}>{err}</span> }
    });

    dom! {
        <div css={base_row_css() + css! {"grid-template-columns: {grid_template_columns};"}}>
            <div css={normal_col_css()}>{section.label.clone()}</div>
            {..fields}
            {..error_node}
            <div>{buttons}</div>
        </div>
    }
}
