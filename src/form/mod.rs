//! This module allows to quickly create a form based on provided model.
//!
//! The model needs to implement converting to [FormData] and from [FormExport],
//! then it can be passed directly to [Form] component.
//!
//! See story book for examples.

use std::rc::Rc;
use vertigo::{bind, component, css, dom, Css};

use crate::{input::NamedInput, DictSelect, DropImageFile, DropImageFileParams, Select};

mod field;
pub use field::{DataFieldValue, FieldExport, FormExport};
mod form_data;
pub use form_data::{DataField, DataSection, FieldsetStyle, FormData};

pub struct FormParams {
    pub css: Css,
    pub add_css: Css,
    pub submit_label: String,
}

impl Default for FormParams {
    fn default() -> Self {
        Self {
            css: css! { "
                display: grid;
                grid-template-rows: auto 1fr;
                gap: 5px;
            " },
            add_css: Css::default(),
            submit_label: "Submit".to_string(),
        }
    }
}

#[component]
pub fn Field<'a>(field: &'a DataField) {
    match &field.value {
        DataFieldValue::String(val) => {
            dom! { <NamedInput name={&field.key} value={val.value.clone()} /> }
        }
        DataFieldValue::List(val) => {
            dom! { <Select value={val.value.clone()} options={&val.options} /> }
        }
        DataFieldValue::Dict(val) => {
            dom! { <DictSelect value={val.value.clone()} options={&val.options} /> }
        }
        DataFieldValue::Image(val) => {
            dom! { <DropImageFile
                item={val.value.clone()}
                original_link={val.original_link.clone()}
                params={DropImageFileParams {
                    img_css: css! { "
                        max-width: 400px;
                        max-height: 350px;
                    "},
                    ..Default::default()
                }}
            /> }
        }
    }
}

/// Renders a form that upon "Save" allows to update a model with new values.
///
/// A model needs to implement conversion to [FormData] and from [FormExport] to interoperate with this component.
///
/// See [FormData] for description how to manage form structure.
#[component]
pub fn Form<'a, T>(model: &'a T, on_submit: Rc<dyn Fn(T)>, params: FormParams)
where
    FormData: From<&'a T>,
    T: From<FormExport> + 'static,
{
    let form_data = Rc::new(FormData::from(model));

    let on_submit = bind!(form_data, || {
        let form_data = form_data.clone();
        on_submit(T::from(form_data.export()));
    });

    let subgrid_css = css! {"
        display: grid;
        grid-template-columns: subgrid;
        grid-column: span 2 / span 2;
    "};

    let fieldset_flex_css = css! {"
        display: flex;
        gap: 5px;
    "};

    let fields = form_data.sections.iter().map(|section| {
        if section.fields.len() > 1 {
            let mut values = vec![];
            for (i, field) in section.fields.iter().enumerate() {
                if section.fieldset_style == FieldsetStyle::Dimensions && i > 0 {
                    values.push(dom! { <span>"x"</span> });
                }
                values.push(dom! { <Field {field} /> });
            }

            let label = &section.label;
            dom! {
                <label css={&subgrid_css}>
                    {label}
                    <div css={&fieldset_flex_css}>
                        {..values}
                    </div>
                </label>
            }
        } else if let Some(field) = section.fields.first() {
            dom! {
                <label css={&subgrid_css}>
                    {&section.label}
                    <Field {field} />
                </label>
            }
        } else {
            dom! { <p /> }
        }
    });

    let form_css = params.css.extend(params.add_css.clone());

    dom! {
        <form css={form_css} on_submit={on_submit}>
            {..fields}
            <input type="submit" value={params.submit_label} />
        </form>
    }
}
