use vertigo::{bind, component, dom, AttrGroup, Value};

/// Input connected to provided `Value<String>`.
#[component]
pub fn Input(value: Value<String>, input: AttrGroup) {
    let on_input = bind!(value, |new_value: String| {
        value.set(new_value);
    });

    dom! {
        <input {value} {on_input} {..input} />
    }
}
