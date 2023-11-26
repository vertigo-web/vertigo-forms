use vertigo::{DomNode, dom, Value};
use vertigo_forms::{Input, InputWithButton};

pub fn input() -> DomNode {
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

    dom! {
        <div>
            {input}
            {input_with_button}
        </div>
    }
}
