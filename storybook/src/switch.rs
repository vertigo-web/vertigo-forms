use vertigo::{Value, DomNode, dom, html_entities};
use vertigo_forms::{Switch, SwitchParams};

pub fn switch() -> DomNode {
    let toggle_value = Value::default();

    dom! {
        <p>
            "Toggle 1: "
            <Switch
                value={&toggle_value}
                params={}
            />
            " " { &toggle_value }
        </p>
        <p>
            "Toggle 2: "
            <Switch
                value={&toggle_value}
                params={SwitchParams {
                    on_symbol: html_entities::Theta.to_string(),
                    off_symbol: html_entities::Omicron.to_string(),
                    ..Default::default()
                }}
            />
            " " { toggle_value }
        </p>
    }
}
