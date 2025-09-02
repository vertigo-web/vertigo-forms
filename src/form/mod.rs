//! This module allows to quickly create a form based on provided model.
//!
//! The model needs to implement converting to [FormData] and from [FormExport],
//! then it can be passed directly to [Form] component.
//!
//! See story book for examples.

use std::rc::Rc;
use vertigo::{AttrGroup, Css, Value, bind, bind_rc, component, css, dom};

use crate::{TabsParams, ValidationErrors};

mod data;
pub use data::*;

mod render;
pub use render::*;

#[derive(Clone)]
pub struct FormParams<T: 'static> {
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
    pub tabs_params: Option<TabsParams>,
}

impl<T: 'static> Default for FormParams<T> {
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
            tabs_params: None,
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
/// Use `s` attribute group to pass anything to underlying section (<label> element) (ex. `s:css="my_styles"`)
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

    let validation_errors = params.validation_errors.clone();

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
                let mut css = ctrl_item_css.clone();
                match oper {
                    Operation::Saving => Some(saving_label.clone()),
                    Operation::Success => Some(saved_label.clone()),
                    Operation::Error(err) => {
                        css += css! {"color: red;"};
                        Some(err)
                    }
                    _ => None,
                }
                .map(|operation_str| dom! { <span {css}>{operation_str}</span> })
            })
        );

        if controls.is_empty() {
            None
        } else {
            let mut css_controls = css!("grid-column: span 2;");
            if let Some(custom_css) = &c_config.css {
                css_controls += custom_css;
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

    let section_css = subgrid_css + params.add_section_css;

    let fields = fields(
        &form_data.sections,
        &s,
        validation_errors.clone(),
        &section_css,
    );

    let tabs = tabs(
        &form_data.tabs,
        &params.tabs_params,
        &s,
        validation_errors.clone(),
        &section_css,
        &params.css.clone(),
    );

    let form_css = params.css + params.add_css;

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
    });

    dom! {
        <form css={form_css} on_submit={on_submit} {..f}>
            {..top_controls}
            {..fields}
            {..tabs}
            {..bottom_controls}
        </form>
    }
}
