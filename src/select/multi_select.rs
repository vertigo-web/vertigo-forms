use vertigo::{Computed, Css, DomNode, Value, bind, bind_rc, css, dom};

/// Select component based on vector of `T` values,
/// which allows to have multiple options selected at once.
///
/// Example:
/// ```
/// use vertigo::{DomNode, dom, Value};
/// use vertigo_forms::MultiSelect;
///
/// let value = Value::new(vec!["foo".to_string()]);
/// let options = Value::new(
///     vec![
///         "foo".to_string(),
///         "bar".to_string(),
///         "baz".to_string(),
///     ]
/// );
///
/// dom! {
///     <MultiSelect
///         value={value.clone()}
///         options={options}
///     />
/// };
/// ```
pub struct MultiSelect<T: Clone> {
    pub value: Value<Vec<T>>,
    pub options: Computed<Vec<T>>,
}

impl<T> MultiSelect<T>
where
    T: Clone + From<String> + PartialEq + ToString + 'static,
{
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(&self) -> DomNode {
        let Self { value, options } = self;
        let toggle = bind_rc!(value, |item: &T| {
            value.change(|value| {
                if let Some(idx) = value.iter().position(|i| i == item) {
                    value.remove(idx);
                } else {
                    value.push(item.clone());
                }
            });
        });

        let list = bind!(
            options,
            toggle,
            value.render_value(move |value| {
                bind!(
                    toggle,
                    options.render_list(
                        |item| item.to_string(),
                        bind!(toggle, |item| {
                            let text_item = item.to_string();
                            let on_click = bind!(toggle, item, |_| toggle(&item));
                            let css = if value.contains(item) {
                                css! {"
                            border-style: inset;
                            font-weight: bold;
                            color: green;
                        "}
                            } else {
                                Css::default()
                            };
                            dom! {
                                <button {css} {on_click}>{text_item}</button>
                            }
                        })
                    )
                )
            })
        );

        let list_css = css! {"
            display: flex;
            flex-wrap: wrap;
        "};

        dom! {
            <div css={list_css}>
                {list}
            </div>
        }
    }
}
