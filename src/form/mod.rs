use std::rc::Rc;
use vertigo::{bind, component, css, dom, Css};

use crate::input::NamedInput;

mod form_data;
pub use form_data::{Field, FormData, FormSection};

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
                grid-template-columns: 100px 1fr;
                gap: 10px;
            " },
            add_css: Css::default(),
            submit_label: "Submit".to_string(),
        }
    }
}

#[component]
pub fn Form<'a, T>(model: &'a T, on_submit: Rc<dyn Fn(T)>, params: FormParams)
where
    FormData: From<&'a T>,
    T: From<Rc<FormData>> + 'static,
{
    let form_data = Rc::new(FormData::from(model));

    let on_submit = bind!(form_data, || {
        let form_data = form_data.clone();
        on_submit(T::from(form_data));
    });

    let fields = form_data.sections.iter().map(|section| {
        if section.fields.len() > 1 {
            let values = section
                .fields
                .iter()
                .map(|field| dom! { <NamedInput name={&field.key} value={field.value.clone()} /> });

            let label = &section.label;
            dom! {
                <label id={label}>{label}</label>
                <fieldset aria-labelledby={label}>
                    {..values}
                </fieldset>
            }
        } else if let Some(field) = section.fields.first() {
            let value = field.value.clone();
            dom! {
                <label for={&field.key}>{&section.label}</label>
                <NamedInput name={&field.key} {value} />
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
