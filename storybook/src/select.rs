use vertigo::{dom, DomNode, Value};
use vertigo_forms::Select;

pub fn select() -> DomNode {
    let value = Value::new("foo".to_string());
    let options = Value::new(vec![
        "foo".to_string(),
        "bar".to_string(),
        "baz".to_string(),
    ]);

    dom! {
        <p>
            "Select value: "
            <Select
                value={value.clone()}
                options={options}
            />
        </p>
        <p>
            "Selected value: " {value}
        </p>
    }
}
