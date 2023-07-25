use vertigo::{DomNode, dom, Value};
use vertigo_forms::Input;

pub fn input() -> DomNode {
    let value = Value::default();
    dom! {
        <p>
            "Enter value: "
            <Input value={value.clone()} />
        </p>
        <p>
            "Entered value: " {value}
        </p>
    }
}
