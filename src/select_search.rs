use either::Either;
use std::{collections::HashMap, hash::Hash};
use vertigo::{
    bind, computed_tuple, css, dom, dom_element, transaction, Computed, DomNode, KeyDownEvent,
    Value,
};

/// Input that searches for entered query in provided item list, based on `HashMap<K, V>`.
pub struct SelectSearch<K, V>
where
    K: Clone,
    V: Clone,
{
    /// Currently selected value
    pub value: Value<K>,
    /// List of possible values
    pub options: Computed<HashMap<K, V>>,
    /// Component behavior/display parameters
    pub params: SelectSearchParams,
}

pub struct SelectSearchParams {
    /// Minimum number of letters to open dropdown
    pub min_chars: usize,
    pub input_title: String,
}

impl Default for SelectSearchParams {
    fn default() -> Self {
        Self {
            min_chars: 3,
            input_title: "Enter phrase".to_string(),
        }
    }
}

impl<K, V> SelectSearch<K, V>
where
    K: Clone + ToString + PartialEq + Eq + Hash + 'static,
    V: Clone + ToString + PartialEq + 'static,
{
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(self) -> DomNode {
        let Self {
            value,
            options,
            params,
        } = self;

        // Filter currently typed by user
        let filter = Value::<Option<String>>::default();
        // Toggle for dropdown visibility
        let dropdown_opened = Value::<bool>::default();
        // Item selected in dropdown using keyboard
        let item_selected = Value::<Option<K>>::default();

        // Items list for dropdown display
        let items = computed_tuple!(options, filter).map(move |(inner_options, inner_filter)| {
            if let Some(inner_filter) = inner_filter {
                let inner_filter = inner_filter.to_lowercase();
                if inner_filter.len() >= params.min_chars {
                    // Filter options
                    inner_options
                        .into_iter()
                        .filter(|(_, opt_value)| {
                            opt_value.to_string().to_lowercase().contains(&inner_filter)
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        });

        let dropdown_css = |visible| {
            let display_value = if visible { "block" } else { "none" };
            css! {"
                display: {display_value};
                position: absolute;
                background-color: white;
                box-shadow: 0px 8px 16px 0px rgba(0, 0, 0, 0.4);
                border: 1px black solid;
                z-index: 1;
            "}
        };

        // Render list
        let list_deps = computed_tuple!(dropdown_opened, items, item_selected);
        let list = bind!(
            value,
            filter,
            dropdown_opened,
            list_deps.render_value(move |(inner_dropdown_opened, inner_items, item_selected)| {
                let item_css = |selected: bool| {
                    let bg_color = if selected { "#ccc" } else { "inherit" };

                    css! {"
                            cursor: pointer;
                            padding: 2px 4px;
                            background-color: {bg_color};

                            :hover {
                                background-color: #ccc;
                            };
                        "}
                };

                let list = dom_element! {
                    <div css={dropdown_css(inner_dropdown_opened)} />
                };

                if inner_dropdown_opened {
                    for (opt_key, opt_value) in &inner_items {
                        // Prevent on blur on input
                        let on_mouse_down = || true;
                        let on_click = bind!(value, filter, dropdown_opened, opt_key, |_| {
                            value.set(opt_key.clone());
                            filter.set(None);
                            dropdown_opened.set(false);
                        });
                        list.add_child(dom! {
                            <div
                                id={opt_key.to_string()}
                                css={item_css(item_selected.as_ref() == Some(opt_key))}
                                {on_mouse_down} {on_click}
                            >
                                {opt_value.to_string()}
                            </div>
                        });
                    }
                }

                list.into()
            })
        );

        // Render input
        let input_deps = computed_tuple!(value, options);
        let input = input_deps.render_value(move |(inner_value, options_inner)| {
            // Displayed value is filter, or value label if no filter typed in
            let displayed_value = filter.to_computed().map(move |inner_filter| {
                if let Some(inner_filter) = inner_filter {
                    inner_filter
                } else {
                    options_inner
                        .get(&inner_value)
                        .map(|val| val.to_string())
                        .unwrap_or_default()
                }
            });

            let on_input = bind!(filter, dropdown_opened, |new_value: String| {
                if new_value.len() >= params.min_chars {
                    dropdown_opened.set(true);
                }
                filter.set(Some(new_value));
            });

            let on_blur = bind!(dropdown_opened, || dropdown_opened.set(false));

            // Make items selectable by keyboard arrows
            let hook_key_down = bind!(
                value,
                item_selected,
                filter,
                options,
                items,
                dropdown_opened,
                |key_down: KeyDownEvent| {
                    if key_down.key == "ArrowDown" || key_down.key == "ArrowUp" {
                        transaction(|ctx| {
                            if filter.get(ctx).is_some() {
                                // Create iterator over dropdown, reversed if arrow up
                                let mut items_iter = {
                                    let iter = items.get(ctx).into_iter();
                                    if key_down.key == "ArrowUp" {
                                        Either::Left(iter.rev())
                                    } else {
                                        Either::Right(iter)
                                    }
                                }
                                .peekable();

                                // Save first element for eventual later use
                                let first_key = items_iter.peek().map(|(key, _)| key).cloned();

                                if let Some(inner_item_selected) = item_selected.get(ctx) {
                                    // If some item already selected, advance
                                    if let Some((next_key, _)) = items_iter
                                        .skip_while(|(opt_key, _)| opt_key != &inner_item_selected)
                                        .nth(1)
                                    {
                                        item_selected.set(Some(next_key));
                                    } else {
                                        // Not found, probably last value was filtered out, just set the first one
                                        item_selected.set(first_key);
                                    }
                                } else if let Some((opt_key, _)) = items_iter.next() {
                                    // If nothing selected just take first one
                                    item_selected.set(Some(opt_key));
                                }
                            }
                        });
                        true
                    } else if key_down.key == "Enter" {
                        transaction(|ctx| {
                            if let Some(item_selected) = item_selected.get(ctx) {
                                // Close dropdown
                                dropdown_opened.set(false);
                                // Set input text to chosen item
                                if let Some(opt_value) = options.get(ctx).get(&item_selected) {
                                    filter.set(Some(opt_value.to_string()));
                                }
                                // Set the value itself
                                value.set(item_selected);
                            }
                        });
                        true
                    } else {
                        false
                    }
                }
            );

            dom! {
                <input
                    required="required"
                    title={&params.input_title}
                    value={displayed_value}
                    {on_input} {on_blur} {hook_key_down}
                />
            }
        });

        let dropdown_css = css! {"
            position: relative;
        "};

        dom! {
            <div css={dropdown_css}>
                {input}
                {list}
            </div>
        }
    }
}
