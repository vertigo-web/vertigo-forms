//! This module allows to quickly create a form based on provided model.
//!
//! The model needs to implement converting to [FormData] and from [FormExport],
//! then it can be passed directly to [Form] component.
//!
//! See story book for examples.

use std::rc::Rc;
use vertigo::{bind, bind_rc, component, css, dom, dom_element, AttrGroup, Css};

use crate::{input::Input, DictSelect, DropImageFile, Select, Switch, SwitchParams};

mod field;
pub use field::{DataFieldValue, ImageValue, FieldExport, FormExport, TextAreaValue};
mod form_data;
pub use form_data::{DataField, DataSection, FieldsetStyle, FormData};

pub struct FormParams {
    pub css: Css,
    pub add_css: Css,
    pub add_section_css: Css,
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
            add_section_css: Css::default(),
            submit_label: "Submit".to_string(),
        }
    }
}

#[component]
pub fn Field<'a>(field: &'a DataField) {
    match &field.value {
        DataFieldValue::String(val) => {
            dom! { <Input input:name={&&field.key} value={val.value.clone()} /> }
        }
        DataFieldValue::TextArea(val) => {
            let on_input = bind!(val.value, |new_value: String| {
                value.set(new_value);
            });
            let el = dom_element! { <textarea name={&&field.key} {on_input}>{&val.value}</textarea> };
            if let Some(rows) = val.rows {
                el.add_attr("rows", rows);
            }
            if let Some(cols) = val.cols {
                el.add_attr("cols", cols);
            }
            el.into()
        }
        DataFieldValue::List(val) => {
            dom! { <Select value={val.value.clone()} options={&val.options} /> }
        }
        DataFieldValue::Bool(val) => {
            dom! {
                <Switch i:name={&&field.key} value={&val.value} params={SwitchParams::checkbox()} />
            }
            // let key = field.key.clone();
            // val.value.render_value(move |checked| {
            //     let el = dom_element! { <input name={&key} type="checkbox" /> };
            //     if checked {
            //         el.add_attr("checked", "checked");
            //     }
            //     el.into()
            // })
        }
        DataFieldValue::Dict(val) => {
            dom! { <DictSelect value={val.value.clone()} options={&val.options} /> }
        }
        DataFieldValue::Image(val) => {
            let params = val.component_params.clone().unwrap_or_default();
            dom! { <DropImageFile
                item={val.value.clone()}
                original_link={val.original_link.clone()}
                {params}
            /> }
        }
        DataFieldValue::Custom(val) => {
            (val.render)()
        }
        DataFieldValue::StaticCustom(render) => {
            render()
        }
    }
}

/// Renders a form for provided model, that upon "Save" allows to update a model with new values.
///
/// A model needs to implement conversion to [FormData] and from [FormExport] to interoperate with this component.
///
/// See [FormData] for description how to manage form structure.
///
/// Use `f` attribute group to pass anything to underlying <form> element (ex. `f:css="my_styles"`)
#[component]
pub fn ModelForm<'a, T>(model: &'a T, on_submit: Rc<dyn Fn(T)>, params: FormParams, f: AttrGroup, s: AttrGroup)
where
    FormData: From<&'a T>,
    T: From<FormExport> + 'static,
{
    let form_data = Rc::new(FormData::from(model));

    let on_submit = bind_rc!(form_data, |form_export: FormExport| {
        on_submit(T::from(form_export));
    });

    let mut form_component = Form {
        form_data,
        on_submit,
        params,
    }
    .into_component();

    form_component.f = f;
    form_component.s = s;

    form_component.mount()
}

/// Renders a form for provided [FormData] that upon "Save" allows to grab updated fields from [FormExport].
///
/// See [FormData] for description how to manage form structure.
///
/// Use `f` attribute group to pass anything to underlying <form> element (ex. `f:css="my_styles"`)
#[component]
pub fn Form(
    form_data: Rc<FormData>,
    on_submit: Rc<dyn Fn(FormExport)>,
    params: FormParams,
    // form attrs
    f: AttrGroup,
    // section attrs
    s: AttrGroup,
) {
    let subgrid_css = css! {"
        display: grid;
        grid-template-columns: subgrid;
        grid-column: span 2 / span 2;
    "};
    let section_css = subgrid_css.clone().extend(params.add_section_css.clone());

    let fieldset_flex_css = css! {"
        display: flex;
        gap: 5px;
    "};

    let fields = form_data.sections.iter().map(|section| {
        let attrs = s.clone();
        let custom_fieldset_css = section.fieldset_css.clone().unwrap_or_else(|| css! {""});

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
                <label css={&section_css} {..attrs}>
                    {label}
                    <div css={&fieldset_flex_css} css={custom_fieldset_css}>
                        {..values}
                    </div>
                </label>
            }
        } else if let Some(field) = section.fields.first() {
            dom! {
                <label css={&section_css} {..attrs}>
                    {&section.label}
                    <Field {field} />
                </label>
            }
        } else {
            dom! { <p /> }
        }
    });

    let on_submit = bind_rc!(form_data, || {
        on_submit(form_data.export());
    });

    let form_css = params.css.extend(params.add_css.clone());

    let top_controls = bind!(
        params.submit_label,
        form_data.top_controls.css,
        form_data.top_controls.submit.then(move || {
            let mut el = dom_element! {
                <div>
                    <input type="submit" value={submit_label} />
                </div>
            };
            if let Some(css) = css {
                el = el.css(css);
            }
            el
        })
    );

    // TODO: Deduplicate
    let bottom_controls = bind!(
        params.submit_label,
        form_data.bottom_controls.css,
        form_data.bottom_controls.submit.then(move || {
            let mut el = dom_element! {
                <div>
                    <input type="submit" value={submit_label} />
                </div>
            };
            if let Some(css) = css {
                el = el.css(css);
            }
            el
        })
    );

    dom! {
        <form css={form_css} on_submit={on_submit} {..f}>
            {..top_controls}
            {..fields}
            {..bottom_controls}
        </form>
    }
}
