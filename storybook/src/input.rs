use vertigo::{dom, DomNode, Value};
use vertigo_forms::{Input, InputWithButton, ListInput};

pub fn input() -> DomNode {
    // Input
    let value = Value::default();
    let input = dom! {
        <h4>"Input"</h4>
        <p>
            "Enter value: "
            <Input value={value.clone()} />
        </p>
        <p>
            "Entered value: " {value}
        </p>
    };

    // Input with button
    let value = Value::default();
    let input_with_button = dom! {
        <h4>"InputWithButton"</h4>
        <p>
            "Enter value: "
            <InputWithButton value={value.clone()} params={} />
        </p>
        <p>
            "Entered value: " {value}
        </p>
    };

    // List input
    let value = Value::default();
    let list_input = dom! {
        <h4>"ListInput"</h4>
        <p>
            "Enter comma-separated values: "
            <ListInput value={value.clone()} />
        </p>
        <p>
            "Entered value: " {value.map(|v| v.join(" :: "))}
        </p>
    };

    dom! {
        <div>
            {input}
            {input_with_button}
            {list_input}
        </div>
    }
}
