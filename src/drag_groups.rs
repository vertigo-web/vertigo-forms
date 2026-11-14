use std::{cell::Cell, rc::Rc};

use vertigo::{
    AttrGroup, ClickEvent, Computed, Css, DomComment, DomNode, KeyDownEvent, PointerEvent, Value,
    bind, component, css, dom,
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
    /// The container of the groups. The component puts `position: relative` before it, for the
    /// copy of the dragged item to stand in it.
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
    /// Added to the button of the item, for the copy of it which follows the pointer while it's
    /// dragged. The copy lets the pointer through, whatever it says.
    pub dragged_css: Css,
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
            dragged_css: css! {"
                opacity: 0.85;
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
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
/// another on "here", in each group it may go to. A second tap, click, Enter or Space on the picked
/// item, or Escape, puts it down - but the picked item can still be dragged.
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
/// Once the pointer has gone a few pixels, a copy of the dragged item follows it (see
/// [`DragGroupsParams::dragged_css`]), held where it was grabbed and kept within the component.
/// It hears the pointer anywhere on the page (`hook_pointer_move`, `hook_pointer_up`), so it goes
/// away also when the item is let go outside of the component - which puts it nowhere.
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
        drag: Value::default(),
        pressed: Cell::new(None),
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
    let on_pointer_down = bind!(parts, |event: PointerEvent| parts.press(&event));
    let css = css! {"position: relative;"} + parts.params.css.clone();
    let drag_layer = parts.render_drag_layer();
    let groups = groups.render_value(move |groups| {
        let groups = groups
            .iter()
            .map(|group| parts.render_group(group))
            .collect();
        DomComment::dom_fragment(groups).into()
    });

    dom! {
        <div css={css} on_key_down={on_key_down} on_pointer_down={on_pointer_down}>
            {groups}
            {drag_layer}
        </div>
    }
}

/// How far, in CSS pixels, the pointer goes from where it pressed the picked item before the item
/// is dragged - and its copy follows the pointer. A tap that moves a little only picks it.
const DRAG_DISTANCE: f64 = 4.0;

/// A position, or a size, in CSS pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn of_pointer(event: &PointerEvent) -> Self {
        Self {
            x: event.pointer_x,
            y: event.pointer_y,
        }
    }

    fn of_element(event: &PointerEvent) -> Self {
        Self {
            x: event.element_width,
            y: event.element_height,
        }
    }
}

/// Where the picked item was pressed, which the item tells the component in the same event.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Press {
    /// From the top left corner of the item.
    grab: Point,
    /// The size of the item.
    size: Point,
    /// The item was picked already: let go without dragging it, the press puts it down.
    was_picked: bool,
}

/// A pointer pressing the picked item, which drags it once it has gone [`DRAG_DISTANCE`].
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drag {
    pointer_id: i32,
    press: Press,
    /// Where the pointer pressed the item, from the top left corner of the component.
    start: Point,
    /// Where the copy of the item stands, from the top left corner of the component; none until
    /// the item is dragged.
    copy_at: Option<Point>,
}

impl Drag {
    /// The pointer is at `at`, in the component of the size `area`: the copy of the item follows
    /// it, held where the item was grabbed, but within the component.
    fn follow(&mut self, at: Point, area: Point) {
        let (dx, dy) = (at.x - self.start.x, at.y - self.start.y);
        if self.copy_at.is_none() && dx.hypot(dy) < DRAG_DISTANCE {
            return;
        }
        let Press { grab, size, .. } = self.press;
        // `min` before `max`: an item bigger than the component sticks to its top left corner
        let within = |at: f64, room: f64| at.min(room).max(0.0);
        self.copy_at = Some(Point {
            x: within(at.x - grab.x, area.x - size.x),
            y: within(at.y - grab.y, area.y - size.y),
        });
    }

    /// Letting go ends a second tap or click on the picked item, which puts it down: it was picked
    /// before the press, and not dragged.
    fn puts_down(&self) -> bool {
        self.press.was_picked && self.copy_at.is_none()
    }
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

/// A pointer presses `this`, which is picked by that, so that it can be dragged at once. True
/// when it was picked already - letting go without dragging it puts it down then.
fn press_on<G: PartialEq, I: PartialEq>(
    held: &mut Option<Picked<G, I>>,
    this: Picked<G, I>,
) -> bool {
    let was_picked = held.as_ref() == Some(&this);
    *held = Some(this);
    was_picked
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

/// The pointer of the drag is lifted, or the browser took it over: the drag ends. What it was, if
/// it's of that pointer.
fn end_drag(drag: &Value<Option<Drag>>, event: &PointerEvent) -> Option<Drag> {
    let mut ended = None;
    drag.change(|drag| {
        if drag.is_some_and(|drag| drag.pointer_id == event.pointer_id) {
            ended = drag.take();
        }
    });
    ended
}

/// What each group is rendered with, anew whenever the groups change.
struct Parts<G: Clone + PartialEq + 'static, I: Clone + PartialEq + 'static> {
    held: Value<Option<Picked<G, I>>>,
    drag: Value<Option<Drag>>,
    /// Where the picked item was pressed, from the item to the component, which starts the drag in
    /// the same event.
    pressed: Cell<Option<Press>>,
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

    fn render_item(self: &Rc<Self>, item: &I, from: &G) -> DomNode {
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

        let parts = self.clone();
        let on_pointer_down = bind!(parts, this, |event: PointerEvent| {
            if event.button == 0 {
                // The browser captured a finger or a pen for the item: it goes on to the group
                // it's lifted over
                event.release_pointer();
                let mut was_picked = false;
                parts
                    .held
                    .change(|held| was_picked = press_on(held, this.clone()));
                // The component, which the event reaches next, starts dragging it
                parts.pressed.set(Some(Press {
                    grab: Point::of_pointer(&event),
                    size: Point::of_element(&event),
                    was_picked,
                }));
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

    /// A pointer pressed the component: it drags the item it pressed, if any.
    fn press(&self, event: &PointerEvent) {
        let drag = self.pressed.take().map(|press| Drag {
            pointer_id: event.pointer_id,
            press,
            start: Point::of_pointer(event),
            copy_at: None,
        });
        self.drag.set(drag);
    }

    /// While a pointer presses the picked item: the layer over the component which hears it
    /// anywhere on the page, with the copy of the item once it's dragged.
    fn render_drag_layer(self: &Rc<Self>) -> DomNode {
        let pressing = self.drag.map(|drag| drag.is_some());
        let parts = self.clone();
        pressing.render_value_option(move |pressing| pressing.then(|| parts.render_dragging()))
    }

    fn render_dragging(&self) -> DomNode {
        let drag = &self.drag;
        // Covers the component, so that the pointer goes from its corner as on the component
        let layer_css = css! {"
            position: absolute;
            inset: 0;
            z-index: 1;
            pointer-events: none;
        "};
        let on_move = bind!(drag, |event: PointerEvent| {
            drag.change(|drag| {
                let Some(this) = drag
                    .as_mut()
                    .filter(|this| this.pointer_id == event.pointer_id)
                else {
                    return;
                };
                if event.buttons & 1 == 0 {
                    // Let go where nothing told the page, f. ex. outside of the window
                    *drag = None;
                } else {
                    this.follow(Point::of_pointer(&event), Point::of_element(&event));
                }
            })
        });
        // After the `on_pointer_up` of the group it's let go over, which puts the item there
        let held = &self.held;
        let on_up = bind!(drag, held, |event: PointerEvent| {
            if end_drag(&drag, &event).is_some_and(|ended| ended.puts_down()) {
                put_down(&held);
            }
        });
        let on_cancel = bind!(drag, |event: PointerEvent| {
            end_drag(&drag, &event);
        });

        // The item, while it's dragged and still picked - not after Escape
        let dragged = Computed::from({
            let (held, drag) = (self.held.clone(), drag.clone());
            move |ctx| match drag.get(ctx) {
                Some(Drag {
                    copy_at: Some(_), ..
                }) => held.get(ctx).map(|held| held.item),
                _ => None,
            }
        });
        // As big as the item, which may be stretched by its group
        let copy_style = drag.map(|drag| match drag {
            Some(Drag {
                copy_at: Some(Point { x, y }),
                press: Press { size, .. },
                ..
            }) => format!(
                "width: {}px; height: {}px; transform: translate({x}px, {y}px)",
                size.x, size.y
            ),
            _ => String::new(),
        });
        let copy_css = css! {"
            position: absolute;
            left: 0;
            top: 0;
            margin: 0;
            box-sizing: border-box;
        "} + self.params.item_css.clone()
            + self.params.dragged_css.clone()
            + css! {"pointer-events: none;"};
        let render_item = self.render_item.clone();
        let copy = dragged.render_value_option(move |item| {
            item.map(|item| {
                dom! {
                    <button
                        type="button"
                        tabindex="-1"
                        aria-hidden="true"
                        css={&copy_css}
                        style={copy_style.clone()}
                    >
                        {render_item(&item)}
                    </button>
                }
            })
        });

        dom! {
            <div
                css={layer_css}
                hook_pointer_move={on_move}
                hook_pointer_up={on_up}
                hook_pointer_cancel={on_cancel}
            >
                {copy}
            </div>
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
    use super::{Drag, DragMove, Picked, Point, Press, pick, press_on, put};

    fn point(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    /// A 40 x 20 item, grabbed 10 px from its left edge and 5 px from its top, pressed at (100, 50)
    /// of the component.
    fn pressed() -> Drag {
        Drag {
            pointer_id: 1,
            press: Press {
                grab: point(10.0, 5.0),
                size: point(40.0, 20.0),
                was_picked: false,
            },
            start: point(100.0, 50.0),
            copy_at: None,
        }
    }

    /// A 500 x 300 component.
    const AREA: Point = Point { x: 500.0, y: 300.0 };

    #[test]
    fn a_pointer_moving_a_little_only_picks_the_item() {
        let mut drag = pressed();

        drag.follow(point(102.0, 51.0), AREA);

        assert_eq!(drag.copy_at, None);
    }

    #[test]
    fn further_the_copy_follows_the_pointer_held_where_it_was_grabbed() {
        let mut drag = pressed();

        drag.follow(point(120.0, 60.0), AREA);
        assert_eq!(drag.copy_at, Some(point(110.0, 55.0)));

        drag.follow(point(101.0, 50.0), AREA);
        assert_eq!(
            drag.copy_at,
            Some(point(91.0, 45.0)),
            "and once dragged, also back to where it was pressed"
        );
    }

    #[test]
    fn the_copy_stays_within_the_component() {
        let mut drag = pressed();

        drag.follow(point(-50.0, 400.0), AREA);
        assert_eq!(drag.copy_at, Some(point(0.0, 280.0)));

        drag.follow(point(1000.0, -10.0), AREA);
        assert_eq!(drag.copy_at, Some(point(460.0, 0.0)));
    }

    #[test]
    fn a_copy_wider_than_the_component_stands_at_its_left_edge() {
        let mut drag = pressed();
        drag.press.size = point(600.0, 20.0);

        drag.follow(point(300.0, 100.0), AREA);

        assert_eq!(drag.copy_at, Some(point(0.0, 95.0)));
    }

    #[test]
    fn a_press_picks_the_item_and_a_press_on_the_picked_one_keeps_it_for_dragging() {
        let mut held = None;

        assert!(!press_on(&mut held, Picked { item: "a", from: 1 }));
        assert_eq!(held, picked("a", 1));

        assert!(press_on(&mut held, Picked { item: "a", from: 1 }));
        assert_eq!(held, picked("a", 1));

        assert!(!press_on(&mut held, Picked { item: "b", from: 2 }));
        assert_eq!(
            held,
            picked("b", 2),
            "and a press on another one picks that one"
        );
    }

    #[test]
    fn let_go_without_dragging_a_second_press_puts_the_item_down() {
        let mut second = pressed();
        second.press.was_picked = true;
        assert!(second.puts_down());

        second.follow(point(120.0, 60.0), AREA);
        assert!(!second.puts_down(), "dragged, it stays picked");

        assert!(!pressed().puts_down(), "and the first press only picks it");
    }

    fn picked(item: &'static str, from: u8) -> Option<Picked<u8, &'static str>> {
        Some(Picked { item, from })
    }

    #[test]
    fn a_key_picks_the_item_and_a_second_one_puts_it_down() {
        let mut held = None;

        pick(&mut held, Picked { item: "a", from: 1 });
        assert_eq!(held, picked("a", 1));

        pick(&mut held, Picked { item: "a", from: 1 });
        assert_eq!(held, None);
    }

    #[test]
    fn a_key_on_another_item_picks_that_one() {
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
