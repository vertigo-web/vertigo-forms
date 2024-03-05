use vertigo::{bind, dom, DomNode, Value};

/// Simple Select component based on vector of T values.
///
/// Example:
/// ```
/// use vertigo::{DomNode, dom, Value};
/// use vertigo_forms::Select;
///
/// let value = Value::new("foo".to_string());
/// let options = Value::new(
///     vec![
///         "foo".to_string(),
///         "bar".to_string(),
///         "baz".to_string(),
///     ]
/// );
///
/// dom! {
///     <Select
///         value={value.clone()}
///         options={options}
///     />
/// };
/// ```
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
