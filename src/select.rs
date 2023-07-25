use vertigo::{bind, dom, DomNode, Value};

/// Simple Select component based on vector of T values.
pub struct Select<T> {
    pub value: Value<T>,
    pub options: Value<Vec<T>>
}

impl<T> Select<T>
where
    T: Clone + From<String> + PartialEq + ToString + 'static
{
    pub fn mount(&self) -> DomNode {
        let Self { value, options } = self;
        let on_change = bind!(value, |new_value: String| {
            value.set(new_value.into());
        });

        let list = bind!(options, value.render_value(move |value|
            options.render_list(
                |item| item.to_string(),
                move |item| {
                    let text_item = item.to_string();
                    if item == &value {
                        dom! { <option value={&text_item} selected="selected">{text_item}</option> }
                    } else {
                        dom! { <option value={&text_item}>{text_item}</option> }
                    }
                }
            )
        ));

        dom! {
            <select {on_change}>
                {list}
            </select>
        }
    }
}
