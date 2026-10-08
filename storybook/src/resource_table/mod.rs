use vertigo::{DomNode, Value, dom};
use vertigo_forms::{Switch, SwitchParams};

mod data_section;
mod fake_server;
mod text_field;

pub fn resource_table() -> DomNode {
    let server_rejecting = Value::new(false);

    dom! {
        <div>
            <p>
                <label>
                    <Switch value={&server_rejecting} params={SwitchParams::checkbox()} />
                    " Server rejects changes"
                </label>
            </p>
            <p>
                "While checked, the server takes a second to reject every change. Until then the
                table shows the change, then it rolls it back and the row returns to where it was,
                with the error: to the form with what was typed in, or to the list."
            </p>
            {text_field::resource_table_text_field(&server_rejecting)}
            {data_section::resource_table_data_section(&server_rejecting)}
        </div>
    }
}
