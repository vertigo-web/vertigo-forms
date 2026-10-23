use std::rc::Rc;

use vertigo::{
    ClickEvent, Computed, Css, DomNode, Value, bind, computed_tuple, css, dom, render::render_list,
    transaction,
};

/// What a click on an option of [MultiSelect] does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MultiSelectMode {
    /// A click adds the option to the selection or takes it away.
    #[default]
    Toggle,
    /// A click selects only the clicked option. With Ctrl or ⌘ held it is added to the selection
    /// or taken away instead, as in a `<select multiple>`.
    Exclusive,
}

impl MultiSelectMode {
    /// Changes `value` as a click on `item` does, with Ctrl or ⌘ held when `modifier`.
    fn click<T: Clone + PartialEq>(self, value: &mut Vec<T>, item: &T, modifier: bool) {
        if self == Self::Exclusive && !modifier {
            *value = vec![item.clone()];
        } else if let Some(idx) = value.iter().position(|i| i == item) {
            value.remove(idx);
        } else {
            value.push(item.clone());
        }
    }
}

#[derive(Clone)]
pub struct MultiSelectParams {
    pub mode: MultiSelectMode,
    /// Label of a button before the options, which clears the selection and is lit while none
    /// of the options is selected - f. ex. "All" of a filter, where nothing chosen means
    /// nothing filtered out.
    pub none_label: Option<String>,
    /// The container of the buttons
    pub list_css: Css,
    /// Every button
    pub button_css: Css,
    /// Added to the button of a selected option
    pub selected_css: Css,
    /// Added to the button of an option not selected
    pub unselected_css: Css,
}

impl Default for MultiSelectParams {
    fn default() -> Self {
        Self {
            mode: MultiSelectMode::default(),
            none_label: None,
            list_css: css! {"
                display: flex;
                flex-wrap: wrap;
            "},
            button_css: Css::default(),
            selected_css: css! {"
                border-style: inset;
                font-weight: bold;
                color: green;
            "},
            unselected_css: Css::default(),
        }
    }
}

/// Select component based on vector of `T` values,
/// which allows to have multiple options selected at once.
///
/// Every option is a button, with `aria-pressed` telling whether it is selected.
///
/// Example:
/// ```
/// use vertigo::{DomNode, dom, Value};
/// use vertigo_forms::{MultiSelect, MultiSelectMode, MultiSelectParams};
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
///         options={options.clone()}
///         params={}
///     />
/// };
///
/// // A filter: a click shows only one option, Ctrl+click adds another
/// dom! {
///     <MultiSelect
///         value={value}
///         options={options}
///         params={MultiSelectParams {
///             mode: MultiSelectMode::Exclusive,
///             none_label: Some("All".to_string()),
///             ..Default::default()
///         }}
///     />
/// };
/// ```
pub struct MultiSelect<T: Clone + PartialEq + 'static> {
    pub value: Value<Vec<T>>,
    pub options: Computed<Vec<T>>,
    pub params: MultiSelectParams,
}

impl<T> MultiSelect<T>
where
    T: Clone + From<String> + PartialEq + ToString + 'static,
{
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(&self) -> DomNode {
        let Self {
            value,
            options,
            params,
        } = self;
        let mode = params.mode;
        let click = {
            let value = value.clone();
            Rc::new(move |item: &T, modifier: bool| {
                value.change(|value| mode.click(value, item, modifier));
            })
        };
        let button_css = {
            let MultiSelectParams {
                button_css,
                selected_css,
                unselected_css,
                ..
            } = params.clone();
            move |selected: bool| {
                button_css.clone().extend(if selected {
                    selected_css.clone()
                } else {
                    unselected_css.clone()
                })
            }
        };

        let none = params.none_label.clone().map(|label| {
            let selected = computed_tuple!(value, options)
                .map(|(value, options)| !options.iter().any(|option| value.contains(option)));
            let on_click = bind!(value, |_| value.set(Vec::new()));
            dom! {
                <button type="button" css={selected.map(button_css.clone())} {on_click}
                        aria-pressed={selected.map(|s| s.to_string())}>
                    {label}
                </button>
            }
        });

        let list = render_list(options, |item| item.to_string(), {
            let value = value.clone();
            move |item: &Computed<T>| {
                let item = item.clone();
                let text_item = item.map(|item| item.to_string());

                let on_click = bind!(click, item, |event: ClickEvent| transaction(|ctx| click(
                    &item.get(ctx),
                    event.ctrl_key || event.meta_key
                )));

                let selected =
                    computed_tuple!(item, value).map(|(item, value)| value.contains(&item));

                dom! {
                    <button type="button" css={selected.map(button_css.clone())} {on_click}
                            aria-pressed={selected.map(|s| s.to_string())}>
                        {text_item}
                    </button>
                }
            }
        });

        dom! {
            <div css={&params.list_css}>
                {..none}
                {list}
            </div>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultiSelectMode;

    fn clicked(mode: MultiSelectMode, value: &[&str], item: &str, modifier: bool) -> Vec<String> {
        let mut value = value.iter().map(|v| v.to_string()).collect();
        mode.click(&mut value, &item.to_string(), modifier);
        value
    }

    #[test]
    fn toggle_adds_and_takes_away() {
        let toggle = MultiSelectMode::Toggle;
        assert_eq!(clicked(toggle, &["foo"], "bar", false), ["foo", "bar"]);
        assert_eq!(clicked(toggle, &["foo", "bar"], "foo", false), ["bar"]);
        assert_eq!(clicked(toggle, &["foo"], "bar", true), ["foo", "bar"]);
    }

    #[test]
    fn exclusive_selects_only_the_clicked_option() {
        let exclusive = MultiSelectMode::Exclusive;
        assert_eq!(clicked(exclusive, &["foo", "bar"], "baz", false), ["baz"]);
        assert_eq!(clicked(exclusive, &["foo", "bar"], "foo", false), ["foo"]);
        assert_eq!(clicked(exclusive, &[], "foo", false), ["foo"]);
    }

    #[test]
    fn exclusive_with_a_modifier_adds_and_takes_away() {
        let exclusive = MultiSelectMode::Exclusive;
        assert_eq!(clicked(exclusive, &["foo"], "bar", true), ["foo", "bar"]);
        assert_eq!(clicked(exclusive, &["foo", "bar"], "foo", true), ["bar"]);
        assert!(clicked(exclusive, &["foo"], "foo", true).is_empty());
    }
}
