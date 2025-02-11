use std::rc::Rc;
use vertigo::{bind, component, css, dom, Css};

use crate::input::NamedInput;

mod form_data;
pub use form_data::{Field, FieldsetStyle, FormData, FormSection};

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
                values.push(dom! { <NamedInput name={&field.key} value={field.value.clone()} /> })
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
            let value = field.value.clone();
            dom! {
                <label css={&subgrid_css}>
                    {&section.label}
                    <NamedInput name={&field.key} {value} />
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
