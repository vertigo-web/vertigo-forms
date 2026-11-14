use std::rc::Rc;

use vertigo::{
    AttrGroup, ClickEvent, Computed, Css, DomNode, KeyDownEvent, PointerEvent, Value, bind,
    component, css, dom,
};

/// Renders what stands above the items of a group of [`DragGroups`], from its key.
pub type RenderGroupHeader<G> = Rc<dyn Fn(&G) -> DomNode>;
/// Renders what stands below the items of a group of [`DragGroups`], from its key, if anything.
pub type RenderGroupFooter<G> = Rc<dyn Fn(&G) -> Option<DomNode>>;
/// Renders what an item of [`DragGroups`] shows.
pub type RenderDragItem<I> = Rc<dyn Fn(&I) -> DomNode>;
/// Attributes of a group of [`DragGroups`] from its key.
pub type DragGroupAttrs<G> = Rc<dyn Fn(&G) -> AttrGroup>;
/// What [`DragGroups`] does with an item put into another group.
pub type OnDragMove<G, I> = Rc<dyn Fn(DragMove<G, I>)>;

/// A group of [`DragGroups`] with its items.
#[derive(Clone, Debug, PartialEq)]
pub struct DragGroup<G, I> {
    /// Tells the group apart; [`DragGroups`] gives it to `on_move` as where an item goes.
    pub key: G,
    /// Its items, in their order. An item stands in one group only.
    pub items: Vec<I>,
}

/// An item put into another group of [`DragGroups`].
#[derive(Clone, Debug, PartialEq)]
pub struct DragMove<G, I> {
    pub item: I,
    pub from: G,
    pub to: G,
}

#[derive(Clone)]
pub struct DragGroupsParams<G> {
    /// What stands below the items of a group, f. ex. a hint. Nothing by default.
    pub render_footer: RenderGroupFooter<G>,
    /// Attributes of the container of a group, f. ex. its `aria-label` or `data-testid`. None by
    /// default.
    pub group_attrs: DragGroupAttrs<G>,
    /// Shown in a group without items, f. ex. "Nobody".
    pub empty_label: Option<String>,
    /// The button which puts the picked item into its group, in every group it may go to.
    pub here_label: String,
    /// The container of the groups.
    pub css: Css,
    /// The container of each group: the header, the items and the footer.
    pub group_css: Css,
    /// Added to a group the picked item may go to: every one but its own.
    pub target_css: Css,
    /// The container of the items of a group, with the empty label and the "here" button.
    pub items_css: Css,
    /// The button of each item. With `touch-action: none`, as by default, a finger drags the item;
    /// without it, a finger starting on an item scrolls the page and a tap only picks the item.
    pub item_css: Css,
    /// Added to the button of the picked item.
    pub picked_css: Css,
    pub here_css: Css,
    pub empty_css: Css,
}

impl<G> Default for DragGroupsParams<G> {
    fn default() -> Self {
        Self {
            render_footer: Rc::new(|_| None),
            group_attrs: Rc::new(|_| AttrGroup::new()),
            empty_label: None,
            here_label: "Here".to_string(),
            css: css! {"
                display: flex;
                flex-direction: column;
                gap: 8px;
            "},
            group_css: css! {"
                display: flex;
                flex-direction: column;
                gap: 4px;
                padding: 4px 8px;
                border: 1px solid #ccc;
                border-radius: 4px;
                user-select: none;
            "},
            target_css: css! {"
                border-style: dashed;
                border-color: #1a73e8;

                :hover {
                    background-color: #e8f0fe;
                };
            "},
            items_css: css! {"
                display: flex;
                flex-wrap: wrap;
                align-items: center;
                gap: 4px;
            "},
            item_css: css! {"
                cursor: grab;
                user-select: none;
                touch-action: none;
            "},
            picked_css: css! {"
                outline: 2px dashed #1a73e8;
                outline-offset: 1px;
            "},
            here_css: css! {"
                border: 1px dashed #1a73e8;
                background: none;
                color: #1a73e8;
                cursor: pointer;
            "},
            empty_css: css! {"
                color: gray;
                font-style: italic;
            "},
        }
    }
}

/// Items of `groups`, each in one of them, put into another group by dragging it there with
/// a mouse, a pen or a finger. Or picked with a tap, a click, Enter or Space, and put with
/// another on "here", in each group it may go to. A second press on the picked item, or Escape,
/// puts it down.
///
/// The component doesn't move anything itself: `on_move` gets the item, where it was and where
/// it goes, and the item moves when `groups` says so - f. ex. at once, changing the `Value` the
/// groups come from, or when a server agrees. `render_header` gives what stands above the items
/// of a group, `render_item` what an item shows, in a button of the component.
///
/// A finger drags an item as a mouse does: the item releases the pointer the browser captured
/// for it (`PointerEvent::release_pointer`), so the finger is lifted over the group it goes to.
/// Meanwhile the browser doesn't scroll the page, as the items have `touch-action: none` (see
/// [`DragGroupsParams::item_css`]).
///
/// Example:
/// ```
/// use std::rc::Rc;
/// use vertigo::{Value, dom};
/// use vertigo_forms::{DragGroup, DragGroups, DragMove, OnDragMove, RenderDragItem, RenderGroupHeader};
///
/// let groups = Value::new(vec![
///     DragGroup { key: "To do", items: vec!["Write".to_string(), "Test".to_string()] },
///     DragGroup { key: "Done", items: vec![] },
/// ]);
///
/// let on_move: OnDragMove<&str, String> = Rc::new({
///     let groups = groups.clone();
///     move |moved: DragMove<&str, String>| {
///         groups.change(|groups| {
///             for group in groups.iter_mut() {
///                 group.items.retain(|item| *item != moved.item);
///             }
///             if let Some(group) = groups.iter_mut().find(|group| group.key == moved.to) {
///                 group.items.push(moved.item);
///             }
///         });
///     }
/// });
///
/// let render_header: RenderGroupHeader<&str> = Rc::new(|key| dom! { <h4>{*key}</h4> });
/// let render_item: RenderDragItem<String> = Rc::new(|item| dom! { <span>{item}</span> });
///
/// dom! {
///     <DragGroups {groups} {render_header} {render_item} {on_move} params={} />
/// };
/// ```
#[component]
pub fn DragGroups<G, I>(
    groups: Computed<Vec<DragGroup<G, I>>>,
    render_header: RenderGroupHeader<G>,
    render_item: RenderDragItem<I>,
    on_move: OnDragMove<G, I>,
    params: DragGroupsParams<G>,
    /// Additional attributes for the button of each item, f. ex. `i:disabled`
    i: AttrGroup,
    /// Additional attributes for each "here" button
    h: AttrGroup,
) where
    G: Clone + PartialEq + 'static,
    I: Clone + PartialEq + 'static,
{
    let held = Value::<Option<Picked<G, I>>>::default();
    let parts = Rc::new(Parts {
        held: held.clone(),
        render_header,
        render_item,
        on_move,
        params,
        item_attrs: i,
        here_attrs: h,
    });

    let on_key_down = bind!(held, |event: KeyDownEvent| {
        event.key == "Escape" && put_down(&held)
    });
    let css = parts.params.css.clone();
    groups.render_value(move |groups| {
        let groups = groups.iter().map(|group| parts.render_group(group));
        dom! { <div css={&css} on_key_down={on_key_down.clone()}>{..groups}</div> }
    })
}

/// The item picked to go to another group, and the group it is in.
#[derive(Clone, Debug, PartialEq)]
struct Picked<G, I> {
    item: I,
    from: G,
}

/// Picks `this`, or puts it down when it's picked already.
fn pick<G: PartialEq, I: PartialEq>(held: &mut Option<Picked<G, I>>, this: Picked<G, I>) {
    *held = (held.as_ref() != Some(&this)).then_some(this);
}

/// Where the picked item goes when it's put into the group `to`; none, and it stays picked,
/// when it's in that group already.
fn put<G: Clone + PartialEq, I>(held: &mut Option<Picked<G, I>>, to: &G) -> Option<DragMove<G, I>> {
    let from = &held.as_ref()?.from;
    if from == to {
        return None;
    }
    let Picked { item, from } = held.take()?;
    Some(DragMove {
        item,
        from,
        to: to.clone(),
    })
}

/// Puts the picked item down; false when none was.
fn put_down<G: Clone + PartialEq + 'static, I: Clone + PartialEq + 'static>(
    held: &Value<Option<Picked<G, I>>>,
) -> bool {
    let mut was = false;
    held.change(|held| was = held.take().is_some());
    was
}

/// What each group is rendered with, anew whenever the groups change.
struct Parts<G: Clone + PartialEq + 'static, I: Clone + PartialEq + 'static> {
    held: Value<Option<Picked<G, I>>>,
    render_header: RenderGroupHeader<G>,
    render_item: RenderDragItem<I>,
    on_move: OnDragMove<G, I>,
    params: DragGroupsParams<G>,
    item_attrs: AttrGroup,
    here_attrs: AttrGroup,
}

impl<G, I> Parts<G, I>
where
    G: Clone + PartialEq + 'static,
    I: Clone + PartialEq + 'static,
{
    fn render_group(self: &Rc<Self>, group: &DragGroup<G, I>) -> DomNode {
        let key = group.key.clone();
        let params = &self.params;

        let target = self.held.map({
            let key = key.clone();
            move |held| held.is_some_and(|held| held.from != key)
        });
        let group_css = {
            let (css, target_css) = (params.group_css.clone(), params.target_css.clone());
            target.map(move |target| {
                if target {
                    css.clone() + target_css.clone()
                } else {
                    css.clone()
                }
            })
        };

        // A mouse, a pen or a finger let go over the group, or a tap on "here"
        let parts = self.clone();
        let on_pointer_up = bind!(parts, key, |event: PointerEvent| if event.button == 0 {
            parts.put(&key)
        });
        let here = target.render_value_option({
            let parts = self.clone();
            let key = key.clone();
            // Enter or Space on "here" - a tap or a click put the item already
            move |target| {
                target.then(|| {
                    let on_click = bind!(parts, key, |_: ClickEvent| parts.put(&key));
                    dom! {
                        <button
                            type="button"
                            css={&parts.params.here_css}
                            on_click={on_click}
                            {..parts.here_attrs.clone()}
                        >
                            {&parts.params.here_label}
                        </button>
                    }
                })
            }
        });

        let items = group
            .items
            .iter()
            .map(|item| self.render_item(item, &key))
            .collect::<Vec<_>>();
        let empty = match (&params.empty_label, items.is_empty()) {
            (Some(label), true) => Some(dom! { <span css={&params.empty_css}>{label}</span> }),
            _ => None,
        };
        let footer = (params.render_footer)(&key);
        let attrs = (params.group_attrs)(&key);

        dom! {
            <div css={group_css} on_pointer_up={on_pointer_up} {..attrs}>
                {(self.render_header)(&key)}
                <div css={&params.items_css}>{..items}{..empty}{here}</div>
                {..footer}
            </div>
        }
    }

    fn render_item(&self, item: &I, from: &G) -> DomNode {
        let held = &self.held;
        let this = Picked {
            item: item.clone(),
            from: from.clone(),
        };
        let picked = held.map({
            let this = this.clone();
            move |held| held.as_ref() == Some(&this)
        });
        let item_css = {
            let (css, picked_css) = (self.params.item_css.clone(), self.params.picked_css.clone());
            picked.map(move |picked| {
                if picked {
                    css.clone() + picked_css.clone()
                } else {
                    css.clone()
                }
            })
        };

        let on_pointer_down = bind!(held, this, |event: PointerEvent| {
            if event.button == 0 {
                // The browser captured a finger or a pen for the item: it goes on to the group
                // it's lifted over
                event.release_pointer();
                held.change(|held| pick(held, this.clone()));
            }
        });
        let on_key_down = bind!(held, this, |event: KeyDownEvent| {
            let pressed = matches!(event.key.as_str(), "Enter" | " ");
            if pressed {
                held.change(|held| pick(held, this.clone()));
            }
            pressed
        });

        dom! {
            <button
                type="button"
                css={item_css}
                aria-pressed={picked.map(|picked| picked.to_string())}
                on_pointer_down={on_pointer_down}
                on_key_down={on_key_down}
                {..self.item_attrs.clone()}
            >
                {(self.render_item)(item)}
            </button>
        }
    }

    /// Puts the picked item into the group `to`, if it's in another one.
    fn put(&self, to: &G) {
        let mut moved = None;
        self.held.change(|held| moved = put(held, to));
        if let Some(moved) = moved {
            (self.on_move)(moved);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DragMove, Picked, pick, put};

    fn picked(item: &'static str, from: u8) -> Option<Picked<u8, &'static str>> {
        Some(Picked { item, from })
    }

    #[test]
    fn a_press_picks_the_item_and_a_second_one_puts_it_down() {
        let mut held = None;

        pick(&mut held, Picked { item: "a", from: 1 });
        assert_eq!(held, picked("a", 1));

        pick(&mut held, Picked { item: "a", from: 1 });
        assert_eq!(held, None);
    }

    #[test]
    fn a_press_on_another_item_picks_that_one() {
        let mut held = picked("a", 1);

        pick(&mut held, Picked { item: "b", from: 2 });

        assert_eq!(held, picked("b", 2));
    }

    #[test]
    fn the_picked_item_goes_into_another_group() {
        let mut held = picked("a", 1);

        let moved = put(&mut held, &2);

        assert_eq!(
            moved,
            Some(DragMove {
                item: "a",
                from: 1,
                to: 2
            })
        );
        assert_eq!(held, None, "and it's put down");
    }

    #[test]
    fn let_go_over_its_own_group_the_item_stays_picked() {
        let mut held = picked("a", 1);

        assert_eq!(put(&mut held, &1), None);
        assert_eq!(held, picked("a", 1), "so a tap picks it, as a click does");
    }

    #[test]
    fn nothing_goes_anywhere_without_a_picked_item() {
        let mut held: Option<Picked<u8, &str>> = None;

        assert_eq!(put(&mut held, &2), None);
    }
}
