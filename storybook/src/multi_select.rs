use vertigo::{DomNode, Value, dom};
use vertigo_forms::MultiSelect;

pub fn multi_select() -> DomNode {
    let value = Value::new(vec!["foo".to_string()]);
    let options = Value::new(vec![
        "foo".to_string(),
        "bar".to_string(),
        "baz".to_string(),
    ]);

    dom! {
        <p>
            "Select value: "
            <MultiSelect
                value={value.clone()}
                options={options}
            />
        </p>
        <p>
            "Selected values: " {value.map(|v| v.join(","))}
        </p>
    }
}
