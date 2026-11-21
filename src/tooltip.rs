use std::{cell::Cell, rc::Rc};

use vertigo::{
    AttrGroup, Computed, Css, DomElementRef, DomNode, JsJson, KeyDownEvent, PointerEvent,
    PointerType, ResizeEvent, Value, component, css, dom, dom_element, js, transaction,
};

/// What [`Tooltip`] looks like.
#[derive(Clone)]
pub struct TooltipParams {
    /// The element the tooltip is about, which is a tab stop of the keyboard too.
    pub container_css: Css,
    /// The bubble with the content. It keeps 8 pixels from the edges of the window, so
    /// a `max-width` wider than `calc(100vw - 16px)` doesn't fit.
    pub css: Css,
    /// Whether the element is a control, f. ex. a button or what a tap picks up to drag: the
    /// bubble shows only while a mouse or a pen is over it, a press does what the control does
    /// and hides the bubble till the pointer leaves, and the element is no tab stop of its own,
    /// so a tap, Enter and Space are the control's.
    pub control: bool,
}

impl Default for TooltipParams {
    fn default() -> Self {
        Self {
            container_css: css! {"
                display: inline-block;
                cursor: help;
            "},
            css: css! {"
                max-width: min(20rem, calc(100vw - 16px));
                padding: 4px 8px;
                border-radius: 4px;
                background: #222;
                color: #fff;
                font-size: 0.875rem;
                line-height: 1.35;
                text-align: left;
                box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
            "},
            control: false,
        }
    }
}

/// The least distance, in CSS pixels, between the bubble of a [`Tooltip`] and the edges of the
/// window.
const MARGIN: f64 = 8.0;

/// The distance, in CSS pixels, between the bubble of a [`Tooltip`] and its element.
const GAP: f64 = 6.0;

/// A short text about an element, shown next to it: when a mouse or a pen is over the element,
/// when a finger taps it, or when the keyboard brings the focus to it, as it is a tab stop.
/// Leaving the element with the pointer, a tap anywhere else, Escape or moving the focus away
/// hides it, and so does a second tap. Enter or Space shows or hides it again.
///
/// The bubble stands under the element, centered on it, or over it when there is no room under
/// it, and keeps within the window, 8 pixels from its edges. It stays in the page while
/// hidden, only out of sight, so a screen reader reads it with the element, and the server
/// renders it too.
///
/// On a control, f. ex. a button, a tap and the keys do what the control does: with
/// [`TooltipParams::control`] the bubble shows only while a mouse or a pen is over it, a press
/// hides it till the pointer leaves, and the element isn't a tab stop of its own.
///
/// The bubble has `position: fixed`, so an ancestor with a `transform` takes it out of place.
///
/// Example:
/// ```
/// use vertigo::dom;
/// use vertigo_forms::Tooltip;
///
/// dom! {
///     <p>
///         "This is a "
///         <Tooltip
///             element={dom! { <b>"cat"</b> }}
///             content={dom! { <span>"Clearly not a dog"</span> }}
///             params={}
///         />
///     </p>
/// };
/// ```
#[component]
pub fn Tooltip(
    element: DomNode,
    content: DomNode,
    params: TooltipParams,
    /// Additional attributes for the container: the element and the bubble
    c: AttrGroup,
    /// Additional attributes for the bubble
    t: AttrGroup,
) {
    let state = Rc::new(State::default());
    let control = params.control;

    let style = Computed::from({
        let state = state.clone();
        move |ctx| bubble_style(state.shown.get(ctx), state.size.get(ctx))
    });
    // The size of the bubble once it's shown, which tells where it fits
    let on_resize = {
        let state = state.clone();
        move |event: ResizeEvent| {
            if transaction(|ctx| state.shown.get(ctx)).is_some() {
                state.size.set(Some(Size {
                    width: event.width,
                    height: event.height,
                }));
            }
        }
    };
    let bubble = dom! {
        <span role="tooltip" css={bubble_css() + params.css} style={style} on_resize={on_resize} {..t}>
            {content}
        </span>
    };
    let container = dom_element! {
        <span css={params.container_css} {..c}>
            {element}
            {bubble}
        </span>
    };
    // A control is the tab stop itself
    let container = if control {
        container
    } else {
        container.attr("tabindex", "0")
    };
    let anchor = container.get_ref();

    let on_pointer_move = {
        let (state, anchor) = (state.clone(), anchor.clone());
        move |event: PointerEvent| {
            // A finger moving over the element scrolls the page, or drags
            if event.pointer_type != PointerType::Touch
                && !state.held.get()
                && state.opener().is_none()
            {
                state.open(&anchor, Opener::Hover);
            }
        }
    };
    let on_mouse_leave = {
        let state = state.clone();
        move || {
            state.held.set(false);
            if state.opener() == Some(Opener::Hover) {
                state.close();
            }
        }
    };
    // A press on a control does what the control does, f. ex. drags it, without the bubble
    let on_pointer_down = {
        let state = state.clone();
        move |_: PointerEvent| {
            if control {
                state.held.set(true);
                state.close();
            } else {
                state.pressed.set(true);
            }
        }
    };
    // Anywhere on the page, after the element heard it: a tap on the element shows or hides the
    // bubble, and a press elsewhere hides it, unless the mouse shows it by hovering
    let on_up = {
        let (state, anchor) = (state.clone(), anchor.clone());
        move |event: PointerEvent| {
            let inside = state.pressed.replace(false);
            if inside && event.pointer_type == PointerType::Touch {
                state.toggle(&anchor, Opener::Tap);
            } else if !inside && state.opener().is_some_and(|by| by != Opener::Hover) {
                state.close();
            }
        }
    };
    let on_cancel = {
        let state = state.clone();
        move |_: PointerEvent| state.pressed.set(false)
    };
    let on_key_down = {
        let (state, anchor) = (state.clone(), anchor.clone());
        move |event: KeyDownEvent| match event.key.as_str() {
            "Escape" if state.opener().is_some() => {
                state.close();
                true
            }
            "Enter" | " " if !control => {
                state.toggle(&anchor, Opener::Key);
                true
            }
            _ => false,
        }
    };
    // The keyboard, or `focus()`; a press of a pointer focuses it too, and has its own way
    let on_focus = {
        let (state, anchor) = (state.clone(), anchor.clone());
        move || {
            if !state.pressed.get() && state.opener().is_none() {
                state.open(&anchor, Opener::Key);
            }
        }
    };
    let on_blur = {
        let state = state.clone();
        move || {
            if state.opener().is_some_and(|by| by != Opener::Hover) {
                state.close();
            }
        }
    };

    let container = container
        .on_pointer_move(on_pointer_move)
        .on_mouse_leave(on_mouse_leave)
        .on_pointer_down(on_pointer_down)
        .hook_pointer_up(on_up)
        .hook_pointer_cancel(on_cancel)
        .on_key_down(on_key_down)
        .on_focus(on_focus)
        .on_blur(on_blur);
    DomNode::from(container)
}

/// The box of the bubble: out of the flow, over the page.
fn bubble_css() -> Css {
    css! {"
        position: fixed;
        z-index: 1000;
        box-sizing: border-box;
    "}
}

/// Hidden, the bubble stays in the page out of sight, for a screen reader; shown, it waits out
/// of sight till its size tells where it fits.
fn bubble_style(shown: Option<Shown>, size: Option<Size>) -> String {
    match (shown, size) {
        (None, _) => "left: 0; top: 0; width: 1px; height: 1px; margin: -1px; padding: 0; \
            border: 0; overflow: hidden; white-space: nowrap; clip-path: inset(50%);"
            .to_string(),
        (Some(_), None) => "left: 0; top: 0; visibility: hidden;".to_string(),
        (Some(shown), Some(size)) => {
            let Point { x, y } = place(shown.anchor, size, shown.window);
            format!("left: {x}px; top: {y}px;")
        }
    }
}

#[derive(Default)]
struct State {
    shown: Value<Option<Shown>>,
    /// The size of the bubble, once shown.
    size: Value<Option<Size>>,
    /// A pointer was pressed on the element, and isn't let go yet.
    pressed: Cell<bool>,
    /// A pointer pressed a control, which keeps the bubble hidden till it leaves.
    held: Cell<bool>,
}

impl State {
    fn opener(&self) -> Option<Opener> {
        transaction(|ctx| self.shown.get(ctx)).map(|shown| shown.by)
    }

    /// Shows the bubble at the element, where it is on the screen now.
    fn open(&self, anchor: &DomElementRef, by: Opener) {
        let Some((anchor, window)) = measure(anchor) else {
            return;
        };
        self.size.set(None);
        self.shown.set(Some(Shown { by, anchor, window }));
    }

    fn close(&self) {
        self.shown.set(None);
    }

    fn toggle(&self, anchor: &DomElementRef, by: Opener) {
        if self.opener().is_some() {
            self.close();
        } else {
            self.open(anchor, by);
        }
    }
}

/// The box of the element on the screen, and the size of the window, without its scrollbars.
fn measure(anchor: &DomElementRef) -> Option<(Rect, Size)> {
    let rect = js! { #anchor.getBoundingClientRect().toJSON() };
    let JsJson::Object(rect) = rect else {
        return None;
    };
    let side = |name: &str| number(rect.get(name)?.clone());
    let anchor = Rect {
        left: side("left")?,
        top: side("top")?,
        right: side("right")?,
        bottom: side("bottom")?,
    };
    let window = Size {
        width: number(js! { document.documentElement.clientWidth })?,
        height: number(js! { document.documentElement.clientHeight })?,
    };
    Some((anchor, window))
}

fn number(json: JsJson) -> Option<f64> {
    match json {
        JsJson::Number(number) => Some(number.as_f64()),
        _ => None,
    }
}

/// How the bubble was shown, which tells what hides it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opener {
    /// A mouse or a pen over the element: leaving it hides the bubble.
    Hover,
    /// A finger: a second tap, or a tap elsewhere.
    Tap,
    /// The focus from the keyboard, Enter or Space: Escape, Enter or Space again, or the focus
    /// going away.
    Key,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Shown {
    by: Opener,
    /// The element on the screen when the bubble was shown.
    anchor: Rect,
    window: Size,
}

/// A box on the screen, in CSS pixels from the top left corner of the window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

/// A size, in CSS pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Size {
    width: f64,
    height: f64,
}

/// A point on the screen, in CSS pixels from the top left corner of the window.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

/// Where the top left corner of a bubble of `size` stands, for the element at `anchor` in the
/// window of the size `window`: centered on the element, [`GAP`] under it, or over it when it
/// doesn't fit under it and there is more room over it, and [`MARGIN`] from the edges of the
/// window at least. A bubble wider than the window sticks to its left edge.
fn place(anchor: Rect, size: Size, window: Size) -> Point {
    let center = (anchor.left + anchor.right) / 2.0;
    // `min` before `max`, so the left edge wins
    let x = (center - size.width / 2.0)
        .min(window.width - MARGIN - size.width)
        .max(MARGIN);
    let room_below = window.height - MARGIN - (anchor.bottom + GAP);
    let room_above = anchor.top - GAP - MARGIN;
    let y = if size.height <= room_below || room_below >= room_above {
        anchor.bottom + GAP
    } else {
        (anchor.top - GAP - size.height).max(MARGIN)
    };
    Point { x, y }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: Size = Size {
        width: 400.0,
        height: 300.0,
    };

    const BUBBLE: Size = Size {
        width: 100.0,
        height: 40.0,
    };

    fn at(left: f64, top: f64) -> Rect {
        Rect {
            left,
            top,
            right: left + 20.0,
            bottom: top + 20.0,
        }
    }

    #[test]
    fn the_bubble_stands_centered_under_the_element() {
        assert_eq!(
            place(at(190.0, 50.0), BUBBLE, WINDOW),
            Point {
                x: 150.0,
                y: 50.0 + 20.0 + GAP
            }
        );
    }

    #[test]
    fn the_bubble_keeps_within_the_sides_of_the_window() {
        let left = place(at(0.0, 50.0), BUBBLE, WINDOW);
        assert_eq!(left.x, MARGIN);
        let right = place(at(380.0, 50.0), BUBBLE, WINDOW);
        assert_eq!(right.x, WINDOW.width - MARGIN - BUBBLE.width);
    }

    #[test]
    fn the_bubble_goes_over_the_element_without_room_under_it() {
        let point = place(at(190.0, 260.0), BUBBLE, WINDOW);
        assert_eq!(point.y, 260.0 - GAP - BUBBLE.height);
    }

    #[test]
    fn the_bubble_stays_under_the_element_with_more_room_there() {
        let tall = Size {
            width: 100.0,
            height: 200.0,
        };
        // 104 px under the element, 66 over it: neither fits, under has more
        let point = place(at(190.0, 80.0), tall, WINDOW);
        assert_eq!(point.y, 80.0 + 20.0 + GAP);
    }

    #[test]
    fn a_bubble_wider_than_the_window_sticks_to_its_left_edge() {
        let wide = Size {
            width: 500.0,
            height: 40.0,
        };
        assert_eq!(place(at(190.0, 50.0), wide, WINDOW).x, MARGIN);
    }

    #[test]
    fn a_hidden_bubble_is_out_of_sight_and_a_shown_one_waits_for_its_size() {
        assert!(bubble_style(None, None).contains("clip-path: inset(50%)"));
        let shown = Shown {
            by: Opener::Hover,
            anchor: at(190.0, 50.0),
            window: WINDOW,
        };
        assert!(bubble_style(Some(shown), None).contains("visibility: hidden"));
        assert_eq!(
            bubble_style(Some(shown), Some(BUBBLE)),
            format!("left: 150px; top: {}px;", 50.0 + 20.0 + GAP)
        );
    }
}
