use vertigo::{
    AttrGroup, Computed, Value, bind, component, computed_tuple, dom, render::render_list,
};

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
#[component]
pub fn DictSelect<T: Clone + From<String> + PartialEq + ToString + 'static>(
    value: Value<i64>,
    options: Computed<Vec<(i64, T)>>,
    select: AttrGroup,
) {
    let on_change = bind!(value, |new_value: String| {
        value.set(new_value.parse().unwrap_or_default());
    });

    // Generate empty option only if initial value does not match any of provided options
    let empty = computed_tuple!(value, options).render_value_option(|(value, options)| {
        options
            .iter()
            .all(|(key, _)| key != &value)
            .then(|| dom! { <option value="" selected="selected" /> })
    });

    let list = render_list(&options, |(key, _)| key.to_string(), {
        let value = value.clone();
        move |item: &Computed<(i64, T)>| {
            let key = item.map(|(key, _)| key);
            let text_item = item.map(|(_, item)| item.to_string());
            let selected = computed_tuple!(key, value)
                .map(|(key, value)| (key == value).then(|| "selected".to_string()));
            dom! { <option value={&key} {selected}>{text_item}</option> }
        }
    });

    dom! {
        <select {on_change} {..select}>
            {empty}
            {list}
        </select>
    }
}
