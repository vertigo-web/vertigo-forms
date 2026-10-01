use vertigo::{DomNode, Value, dom};
use vertigo_forms::MultiDropDown;

pub fn multi_drop_down() -> DomNode {
    let value = Value::new(vec!["foo".to_string()]);
    let options = Value::new(vec![
        "foo".to_string(),
        "bar".to_string(),
        "baz".to_string(),
    ]);

    dom! {
        <div>
            "Select value: "
            <MultiDropDown
                value={value.clone()}
                options={options}
                params={}
            />
        </div>
        <p>
            "Selected values: " {value.map(|v| v.join(","))}
        </p>
    }
}
