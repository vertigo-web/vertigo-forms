use vertigo::{bind, component, computed_tuple, dom, transaction, AttrGroup, Value};

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
///                 button_label: "Load".to_string(),
///                 ..Default::default()
///         }}
///         input:css={css!("width: 300px;")}
///     />
/// };
/// ```
#[component]
pub fn InputWithButton(
    value: Value<String>,
    params: InputWithButtonParams,
    input: AttrGroup,
    button: AttrGroup,
) {
    let temp_value = Value::<Option<String>>::default();
    let display_value =
        computed_tuple!(value, temp_value).map(|(value, temp_value)| temp_value.unwrap_or(value));

    let on_input = bind!(temp_value, |new_value: String| {
        temp_value.set(Some(new_value));
    });

    let on_click = bind!(value, temp_value, |_| {
        transaction(|ctx| {
            let new_value = temp_value.get(ctx);
            if let Some(new_value) = new_value {
                value.set(new_value);
            }
            temp_value.set(None);
        })
    });

    dom! {
        <input value={display_value} {on_input} {..input}/>
        <button {on_click} {..button}>{params.button_label}</button>
    }
}

#[derive(Clone)]
pub struct InputWithButtonParams {
    pub button_label: String,
}

impl Default for InputWithButtonParams {
    fn default() -> Self {
        Self {
            button_label: "OK".to_string(),
        }
    }
}
