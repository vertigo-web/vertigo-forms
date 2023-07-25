use vertigo::{bind, dom, transaction, Value, DomNode};

pub struct SwitchParams {
    pub on_symbol: String,
    pub off_symbol: String,
}

impl Default for SwitchParams {
    fn default() -> Self {
        Self {
            on_symbol: "ON".to_string(),
            off_symbol: "OFF".to_string(),
        }
    }
}

/// Simple toggle control connected to `Value<bool>`.
pub struct Switch {
    pub value: Value<bool>,
    pub params: SwitchParams,
}

impl Switch {
    pub fn mount(self) -> DomNode {
        let Self { value, params } = self;

        let toggle = bind!(value,
            || transaction(|ctx| value.set(!value.get(ctx)))
        );

        let symbol = value.map(move |value|
            if value { params.on_symbol.clone() } else { params.off_symbol.clone() }
        );

        dom! {
            <button on_click={toggle}>{symbol}</button>
        }
    }
}
