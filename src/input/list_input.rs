use vertigo::{AttrGroup, Value, bind, component, dom};

/// Input connected to provided `Value<Vec<String>>`.
///
/// It parsed a comma-separated input string and sets value as a vector.
#[component]
pub fn ListInput(value: Value<Vec<String>>, input: AttrGroup) {
    let value_str = value.map(|v| v.join(","));

    let on_input = bind!(value, |new_value: String| {
        value.set(
            new_value
                .split(',')
                .filter_map(|v| {
                    let v = v.trim();
                    if v.is_empty() {
                        None
                    } else {
                        Some(v.to_string())
                    }
                })
                .collect(),
        );
    });

    dom! {
        <input value={value_str} on_input={on_input} {..input} />
    }
}
