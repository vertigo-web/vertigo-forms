use vertigo::{DomNode, Value, dom};
use vertigo_forms::{MultiSelect, MultiSelectMode, MultiSelectParams};

pub fn multi_select() -> DomNode {
    let value = Value::new(vec!["foo".to_string()]);
    let options = Value::new(vec![
        "foo".to_string(),
        "bar".to_string(),
        "baz".to_string(),
    ]);

    let filter = Value::new(vec!["red".to_string()]);
    let colors = Value::new(vec![
        "red".to_string(),
        "green".to_string(),
        "blue".to_string(),
    ]);

    dom! {
        <div>
            "Select value: "
            <MultiSelect
                value={value.clone()}
                options={options}
                params={}
            />
        </div>
        <p>
            "Selected values: " {value.map(|v| v.join(","))}
        </p>
        <div>
            "Filter, Ctrl+click for more: "
            <MultiSelect
                value={filter.clone()}
                options={colors}
                params={MultiSelectParams {
                    mode: MultiSelectMode::Exclusive,
                    none_label: Some("All".to_string()),
                    ..Default::default()
                }}
            />
        </div>
        <p>
            "Filtered by: " {filter.map(|v| if v.is_empty() { "all".to_string() } else { v.join(",") })}
        </p>
    }
}
