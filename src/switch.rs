use vertigo::{AttrGroup, bind, component, dom, transaction, Value};

pub enum DisplayType {
    Button,
    CheckBox,
}

pub struct SwitchParams {
    pub display_type: DisplayType,
    pub on_symbol: String,
    pub off_symbol: String,
}

impl Default for SwitchParams {
    fn default() -> Self {
        Self {
            display_type: DisplayType::Button,
            on_symbol: "ON".to_string(),
            off_symbol: "OFF".to_string(),
        }
    }
}

impl SwitchParams {
    pub fn checkbox() -> Self {
        Self {
            display_type: DisplayType::CheckBox,
            on_symbol: "".to_string(),
            off_symbol: "".to_string(),
        }
    }
}

/// Simple toggle control connected to `Value<bool>`.
///
/// Example:
/// ```
/// use vertigo::{Value, dom};
/// use vertigo_forms::{Switch, SwitchParams};
///
/// let toggle_value = Value::new(false);
/// dom! {
///     <Switch
///         value={&toggle_value}
///         params={}
///     />
/// };
/// ```
#[component]
pub fn Switch(
    value: Value<bool>,
    params: SwitchParams,
    i: AttrGroup, // TODO: Use
) {
    let toggle = bind!(value, |_| transaction(|ctx| value.set(!value.get(ctx))));

    match params.display_type {
        DisplayType::Button => {
            let symbol = value.map(move |value| {
                if value {
                    params.on_symbol.clone()
                } else {
                    params.off_symbol.clone()
                }
            });

            dom! {
                <button on_click={toggle} {..i}>{symbol}</button>
            }
        }
        DisplayType::CheckBox => {
            let value_clone = value.clone();
            value.render_value(move |value_inner| {
                let toggle = bind!(value_clone, |_| transaction(
                    |ctx| value_clone.set(!value_clone.get(ctx))
                ));
                if value_inner {
                    dom! {
                        <input type="checkbox" on_click={toggle} checked="checked" />
                    }
                } else {
                    dom! {
                        <input type="checkbox" on_click={toggle} />
                    }
                }
            })

            // Following doesn't work as browsers reads attribute 'checked' only on first render
            // let checked = value.map(move |value|
            //     if value { Some("checked".to_string()) } else { None }
            // );
            // dom! {
            //     <input type="checkbox" on_click={toggle} checked={checked} />
            // }
        }
    }
}
