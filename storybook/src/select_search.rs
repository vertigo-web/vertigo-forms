use std::collections::HashMap;
use vertigo::{computed_tuple, dom, DomNode, Value};
use vertigo_forms::SelectSearch;

pub fn select_search() -> DomNode {
    let value = Value::new("");
    let options = Value::new(HashMap::from([
        (
            "well done",
            "Well done is better than well said".to_string(),
        ),
        (
            "once",
            "Once you choose hope, anything's possible".to_string(),
        ),
        (
            "try it",
            "Try it again. Fail again. Fail better".to_string(),
        ),
        (
            "start wide",
            "Start wide, expand further, and never look back".to_string(),
        ),
        (
            "yolo",
            "You only live once but if you do it right, once is enough".to_string(),
        ),
    ]));

    let selected_value = computed_tuple!(value, options)
        .map(|(value, options)| options.get(&value).cloned().unwrap_or_default());

    let hints = options.render_list(
        |(key, _)| key.to_string(),
        |(_key, value)| {
            dom! {
                <li>{value.split(' ').next().unwrap_or_default()}</li>
            }
        },
    );

    dom! {
        <p>
            "Select value: "
            <SelectSearch
                value={value.clone()}
                options={options}
                params={}
            />
        </p>
        <p>"Selected key: " {value}</p>
        <p>"Selected value: " {selected_value}</p>
        <h4>"Hints:"</h4>
        <ul>{hints}</ul>
    }
}
