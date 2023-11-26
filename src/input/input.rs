use vertigo::{bind, dom, DomNode, Value};

/// Input connected to provided `Value<String>`.
pub struct Input {
    pub value: Value<String>,
}

impl Input {
    pub fn mount(self) -> DomNode {
        let Self { value } = self;

        let on_input = bind!(value, |new_value: String| {
            value.set(new_value);
        });

        dom! {
            <input value={value.to_computed()} on_input={on_input} />
        }
    }
}
