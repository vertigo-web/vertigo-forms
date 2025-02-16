use vertigo::{bind, dom, DomNode, Value};

/// Input connected to provided `Value<String>`.
pub struct Input {
    pub value: Value<String>,
}

impl Input {
    pub fn mount(self) -> DomNode {
        let Self { value } = self;

        dom! {
            <NamedInput name="" {value} />
        }
    }
}

/// Input connected to provided `Value<String>` with name defined.
pub struct NamedInput {
    pub name: String,
    pub value: Value<String>,
}

impl NamedInput {
    pub fn mount(self) -> DomNode {
        let Self { name, value } = self;

        let on_input = bind!(value, |new_value: String| {
            value.set(new_value);
        });

        dom! {
            <input {name} {value} {on_input} />
        }
    }
}
