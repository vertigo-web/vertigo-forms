use vertigo::{bind, dom, Computed, DomNode, Value};

/// Simple Select component based on map of `i64`->`T` values.
///
/// Example:
/// ```
/// use vertigo::{DomNode, dom, Value};
/// use vertigo_forms::DictSelect;
///
/// let value = Value::new(1);
/// let options = Value::new(
///     vec![
///         (1, "foo".to_string()),
///         (2, "bar".to_string()),
///         (3, "baz".to_string()),
///     ]
/// );
///
/// dom! {
///     <DictSelect
///         value={value.clone()}
///         options={options}
///     />
/// };
/// ```
pub struct DictSelect<T: Clone> {
    pub value: Value<i64>,
    pub options: Computed<Vec<(i64, T)>>,
}

impl<T> DictSelect<T>
where
    T: Clone + From<String> + PartialEq + ToString + 'static,
{
    pub fn mount(&self) -> DomNode {
        let Self { value, options } = self;
        let on_change = bind!(value, |new_value: String| {
            value.set(new_value.parse().unwrap_or_default());
        });

        let list = bind!(
            options,
            value.render_value(move |value| options.render_list(
                |(key, _)| key.to_string(),
                move |(key, item)| {
                    let text_item = item.to_string();
                    if key == &value {
                        dom! { <option value={&key} selected="selected">{text_item}</option> }
                    } else {
                        dom! { <option value={&key}>{text_item}</option> }
                    }
                }
            ))
        );

        dom! {
            <select {on_change}>
                {list}
            </select>
        }
    }
}
