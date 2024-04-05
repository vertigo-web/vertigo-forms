use vertigo::{bind, computed_tuple, dom, transaction, Css, DomNode, Value};

/// Input connected to provided `Value<String>`.
///
/// ```
/// use vertigo::{dom, Value};
/// use vertigo_forms::InputWithButton;
///
/// let my_value = Value::<String>::default();
///
/// dom! {
///     <InputWithButton value={my_value} params={} />
/// };
/// ```
///
/// ```
/// use vertigo::{css, dom, Value};
/// use vertigo_forms::{InputWithButton, InputWithButtonParams};
///
/// let my_value = Value::new("Initial text".to_string());
///
/// dom! {
///     <InputWithButton
///         value={my_value}
///         params={InputWithButtonParams {
///                 input_css: css!("width: 300px;"),
///                 button_label: "Load".to_string(),
///                 ..Default::default()
///         }}
///     />
/// };
/// ```
pub struct InputWithButton {
    pub value: Value<String>,
    pub params: InputWithButtonParams,
}

#[derive(Clone)]
pub struct InputWithButtonParams {
    pub input_css: Css,
    pub button_label: String,
    pub button_css: Css,
}

impl Default for InputWithButtonParams {
    fn default() -> Self {
        Self {
            input_css: Css::default(),
            button_label: "OK".to_string(),
            button_css: Css::default(),
        }
    }
}

impl InputWithButton {
    pub fn mount(self) -> DomNode {
        let Self { value, params } = self;

        let temp_value = Value::<Option<String>>::default();
        let display_value = computed_tuple!(value, temp_value)
            .map(|(value, temp_value)| temp_value.unwrap_or(value));

        let on_input = bind!(temp_value, |new_value: String| {
            temp_value.set(Some(new_value));
        });

        let on_click = bind!(value, temp_value, || {
            transaction(|ctx| {
                let new_value = temp_value.get(ctx);
                if let Some(new_value) = new_value {
                    value.set(new_value);
                }
                temp_value.set(None);
            })
        });

        dom! {
            <input css={params.input_css} value={display_value} {on_input} />
            <button css={params.button_css} {on_click}>{params.button_label}</button>
        }
    }
}
