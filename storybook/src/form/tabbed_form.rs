use std::rc::Rc;

use vertigo::{Value, bind_rc, component, css, dom};
use vertigo_forms::form::{
    DataFieldValue, DataSection, FormData, FormExport, FormParams, ModelForm, TextAreaValue,
};

use crate::bordered_tabs;

// Tabbed Form example

#[derive(Clone, PartialEq)]
pub struct TModel {
    pub first_name: String,
    pub last_name: String,
    pub annotation: Option<String>,
}

impl From<TModel> for FormData {
    fn from(value: TModel) -> Self {
        Self::default()
            .add_tab(
                "Basic",
                vec![
                    DataSection::with_string_field("First name", "first_name", &value.first_name),
                    DataSection::with_string_field("Last name", "last_name", &value.last_name),
                ],
            )
            .add_tab(
                "Other",
                vec![DataSection::new("Annotation").add_field(
                    "annotation",
                    DataFieldValue::TextArea(TextAreaValue {
                        value: Value::new(value.annotation.clone().unwrap_or_default()),
                        original_value: value.annotation.clone().map(Rc::new),
                        rows: Some(10),
                        cols: None,
                    }),
                )],
            )
            .add_top_controls()
    }
}

impl From<FormExport> for TModel {
    fn from(form_export: FormExport) -> Self {
        Self {
            first_name: form_export.get_string("first_name"),
            last_name: form_export.get_string("last_name"),
            annotation: form_export.get_string_opt("annotation"),
        }
    }
}

#[component]
pub fn TabbedForm() {
    let model: Value<TModel> = Value::new(TModel {
        first_name: "Johann".to_string(),
        last_name: "Gambolputty".to_string(),
        annotation: None,
    });

    let on_submit = bind_rc!(model, |new_model: TModel| {
        model.set(new_model);
    });

    dom! {
        <div>
            <h4>"Tabbed Form:"</h4>
            <ModelForm {model} {on_submit}
                params={FormParams {
                    add_css: css! {"width: 400px;"},
                    tabs_params: Some(bordered_tabs()),
                    ..Default::default()
                }}
            />
        </div>
    }
}
