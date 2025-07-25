//! This module allows to quickly create a form based on provided model.
//!
//! The model needs to implement converting to [FormData] and from [FormExport],
//! then it can be passed directly to [Form] component.
//!
//! See story book for examples.

use std::rc::Rc;
use vertigo::{AttrGroup, Css, Value, bind, bind_rc, component, css, dom, dom_element};

use crate::{
    DictSelect, DropImageFile, Select, Switch, SwitchParams, ValidationErrors, input::Input,
};

mod field;
pub use field::{DataFieldValue, FieldExport, FormExport, ImageValue, TextAreaValue};
mod form_data;
pub use form_data::{ControlsConfig, DataField, DataSection, FieldsetStyle, FormData};

pub type ValidateFunc<T> = Rc<dyn Fn(&T, Value<ValidationErrors>) -> bool>;

#[derive(Default, Clone, PartialEq)]
pub enum Operation {
    #[default]
    None,
    Saving,
    Success,
    Error(String),
}

#[derive(Clone)]
pub struct FormParams<T>
where
    T: From<FormExport> + 'static,
{
    pub css: Css,
    pub add_css: Css,
    pub add_section_css: Css,
    pub submit_label: Rc<String>,
    pub on_delete: Option<Rc<dyn Fn()>>,
    pub delete_label: Rc<String>,
    pub validate: Option<ValidateFunc<T>>,
    pub validation_errors: Value<ValidationErrors>,
    pub operation: Value<Operation>,
    pub saving_label: Rc<String>,
    pub saved_label: Rc<String>,
}

impl<T> Default for FormParams<T>
where
    T: From<FormExport> + 'static,
{
    fn default() -> Self {
        Self {
            css: css! { "
                display: grid;
                grid-template-rows: auto 1fr;
                gap: 5px;
            " },
            add_css: Css::default(),
            add_section_css: Css::default(),
            submit_label: Rc::new("Submit".to_string()),
            on_delete: None,
            delete_label: Rc::new("Delete".to_string()),
            validate: None,
            validation_errors: Default::default(),
            operation: Default::default(),
            saving_label: Rc::new("Saving...".to_string()),
            saved_label: Rc::new("Saved".to_string()),
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
            let el =
                dom_element! { <textarea name={&&field.key} {on_input}>{&val.value}</textarea> };
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
        DataFieldValue::Custom(val) => (val.render)(),
        DataFieldValue::StaticCustom(render) => render(),
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
pub fn ModelForm<'a, T>(
    model: &'a T,
    on_submit: Rc<dyn Fn(T)>,
    params: FormParams<T>,
    f: AttrGroup,
    s: AttrGroup,
) where
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
pub fn Form<T>(
    form_data: Rc<FormData>,
    on_submit: Rc<dyn Fn(FormExport)>,
    params: FormParams<T>,
    // form attrs
    f: AttrGroup,
    // section attrs
    s: AttrGroup,
) where
    T: From<FormExport> + 'static,
{
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

    let validation_errors = params.validation_errors.clone();

    let fields = form_data.sections.iter().flat_map(|section| {
        let attrs = s.clone();
        let custom_fieldset_css = section.fieldset_css.clone().unwrap_or_else(|| css! {""});

        if section.fields.len() > 1 {
            let mut values = vec![];
            for (i, field) in section.fields.iter().enumerate() {
                if section.fieldset_style == FieldsetStyle::Dimensions && i > 0 {
                    values.push(dom! { <span>"x"</span> });
                }
                let val_error = {
                    let field_key = field.key.to_owned();
                    validation_errors.render_value_option(move |errs| {
                        errs.get(&field_key).map(|err| dom! { <span>{err}</span> })
                    })
                };
                values.push(dom! {
                    <div css={css!{"display: flex; flex-flow: column nowrap;"}}>
                        <Field {field} />
                        <span css={css!{"color: red;"}}>{val_error}</span>
                    </div>
                });
            }

            let label = &section.label;

            let section_rendered = dom! {
                <label css={&section_css} {..attrs}>
                    {label}
                    <div css={&fieldset_flex_css} css={custom_fieldset_css}>
                        {..values}
                    </div>
                </label>
            };
                // {..section.new_group.then(|| dom! { <hr css={css!{"width: 100%;"}}/> })}

            if section.new_group {
                vec![
                    dom! {
                        <hr css={css!{"width: 100%; grid-column: 1 / 3;"}}/>
                    },
                    section_rendered
                ]
            } else {
                vec![
                    section_rendered
                ]
            }
        } else if let Some(field) = section.fields.first() {
            let val_error = {
                let field_key = field.key.to_owned();
                validation_errors.render_value_option(move |errs| {
                    errs.get(&field_key).map(|err| dom! { <span>{err}</span> })
                })
            };
            vec![dom! {
                <label css={&section_css} {..attrs}>
                    {&section.label}
                    <div css={css!{"display: flex; flex-flow: column nowrap;"}}>
                        <Field {field} />
                        <span css={css!{"color: red;"}}>{val_error}</span>
                    </div>
                </label>
            }]
        } else {
            vec![dom! { <p /> }]
        }
    });

    let controls = |params: &FormParams<T>, c_config: &ControlsConfig| {
        let mut controls = vec![];

        let ctrl_item_css = css! {"
            margin: 5px;
        "};

        if c_config.submit {
            controls.push(dom! {
                <input css={&ctrl_item_css} type="submit" value={&params.submit_label} />
            });
        }
        if c_config.delete
            && let Some(on_click) = params.on_delete.clone()
        {
            controls.push(dom! {
                <input css={&ctrl_item_css} type="submit" value={&params.delete_label} on_click={move |_| on_click()} />
            });
        }

        let errors = validation_errors
            .render_value_option(|errs| errs.get("submit").map(|err| dom! { <span>{err}</span> }));

        let operation_str = bind!(
            params.saving_label,
            params.saved_label,
            params.operation.render_value_option(move |oper| {
                match oper {
                    Operation::Saving => Some(&saving_label),
                    Operation::Success => Some(&saved_label),
                    _ => None,
                }
                .map(|operation_str| dom! { <span css={&ctrl_item_css}>{operation_str}</span> })
            })
        );

        if controls.is_empty() {
            None
        } else {
            let mut css_controls = css!("grid-column: span 2;");
            if let Some(custom_css) = &c_config.css {
                css_controls = css_controls.extend(custom_css.clone());
            }
            Some(dom! {
                <div css={css_controls}>
                    {..controls}
                    {errors}
                    {operation_str}
                </div>
            })
        }
    };

    let top_controls = controls(&params, &form_data.top_controls);
    let bottom_controls = controls(&params, &form_data.bottom_controls);

    let on_submit = bind_rc!(form_data, validation_errors, || {
        params.operation.set(Operation::Saving);
        let model = form_data.export();
        let valid = if let Some(validate) = &params.validate {
            validate(&model.clone().into(), validation_errors.clone())
        } else {
            true
        };
        if valid {
            on_submit(model);
        }
        // params.operation.change(|oper|
        //     // Auto change operation value only if it was not modified by callback
        //     if oper == &Operation::Saving {
        //         *oper = Operation::Success;
        //     }
        // );
    });

    let form_css = params.css.extend(params.add_css.clone());

    dom! {
        <form css={form_css} on_submit={on_submit} {..f}>
            {..top_controls}
            {..fields}
            {..bottom_controls}
        </form>
    }
}
