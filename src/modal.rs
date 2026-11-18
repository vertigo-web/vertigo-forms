use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use vertigo::{
    AttrGroup, ClickEvent, Css, DomElementRef, DomNode, DropResource, JsJson, KeyDownEvent,
    PointerEvent, Value, WheelEvent, bind, component, css, dom, dom_element, get_driver, js,
};

/// Renders what [`Modal`] shows, anew each time it opens.
pub type RenderModalContent = Rc<dyn Fn() -> DomNode>;

/// What [`Modal`] looks like.
#[derive(Clone)]
pub struct ModalParams {
    /// The layer over the whole page, with the window in the middle. A click on it closes the
    /// window.
    pub overlay_css: Css,
    /// The window: the title over the body, and the close button in the top right corner. It is
    /// no taller than the layer, and the body scrolls when the content is taller.
    pub window_css: Css,
    pub title_css: Css,
    /// The body of the window, with the content.
    pub body_css: Css,
    /// How the close button looks. Where it is is up to [`ModalParams::window_css`]: the window
    /// has `position: relative`, the button `position: absolute`.
    pub close_css: Css,
    /// Title of the close button, shown on hover and read out by screen readers.
    pub close_label: String,
}

impl Default for ModalParams {
    fn default() -> Self {
        Self {
            overlay_css: css! {"
                position: fixed;
                inset: 0;
                z-index: 1000;
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 16px;
                box-sizing: border-box;
                background-color: rgba(0, 0, 0, 0.5);
            "},
            window_css: css! {"
                width: min(40rem, 100%);
                max-height: 100%;
                box-sizing: border-box;
                border-radius: 6px;
                background-color: white;
                color: #222;
                box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
            "},
            title_css: css! {"
                margin: 0;
                padding: 16px 56px 8px 16px;
                font-size: 1.25rem;
            "},
            body_css: css! {"
                padding: 8px 16px 16px;
            "},
            close_css: css! {"
                top: 8px;
                right: 8px;
                width: 40px;
                height: 40px;
                padding: 0;
                border: none;
                border-radius: 50%;
                background-color: transparent;
                color: inherit;
                font-size: 28px;
                line-height: 1;
                cursor: pointer;

                :hover {
                    background-color: rgba(0, 0, 0, 0.08);
                };
            "},
            close_label: "Close".to_string(),
        }
    }
}

/// A window over the whole page, with `title` over what `content` renders, while `open` is
/// `true`, and nothing otherwise. The content is rendered anew each time the window opens.
///
/// Escape, the close button or a click beside the window set `open` to `false`. A press in the
/// window let go beside it, f. ex. selecting text in a field, doesn't. While the window is open,
/// the page under it doesn't scroll, and the keyboard stays in the window: Tab past its last
/// element comes back to the first, and Shift+Tab past the first goes to the last, the close
/// button. A screen reader reads it as a dialog named by its title. Closed, the window gives the
/// focus back to the element that had it when the window opened, f. ex. the button opening it.
///
/// A `Modal` may open another from its content, f. ex. to confirm something: Escape closes the
/// one on top, and the page scrolls again once the last one is closed. An element of the content
/// handling Escape itself, returning `true` from its `on_key_down`, keeps the window open.
///
/// Attributes of `w` go to the window, f. ex. `w:id="settings"`.
///
/// The layer has `position: fixed`, so no ancestor of the component may have a `transform` or
/// a `filter`, which would make it fixed to that ancestor instead of the window.
///
/// Example:
/// ```
/// use std::rc::Rc;
/// use vertigo::{Value, bind, dom};
/// use vertigo_forms::{Modal, RenderModalContent};
///
/// let open = Value::new(false);
/// let show = bind!(open, |_| open.set(true));
/// let content: RenderModalContent = Rc::new(|| dom! { <p>"Clearly not a dog."</p> });
/// dom! {
///     <button on_click={show}>"Show the cat"</button>
///     <Modal {open} title="The cat" {content} params={} />
/// };
/// ```
#[component]
pub fn Modal(
    open: Value<bool>,
    title: String,
    content: RenderModalContent,
    params: ModalParams,
    /// Additional attributes for the window
    w: AttrGroup,
) {
    open.to_computed().render_value_option(move |shown| {
        shown.then(|| render_window(&open, &title, &content, &params, &w))
    })
}

fn render_window(
    open: &Value<bool>,
    title: &str,
    content: &RenderModalContent,
    params: &ModalParams,
    w: &AttrGroup,
) -> DomNode {
    let close = bind!(open, || open.set(false));
    let number = next_number();
    let title_id = format!("vertigo-forms-modal-title-{number}");
    // Before the window takes the focus, which it does once it's on the page
    let focus_back = get_driver().remember_focus();

    let close_css = css! {"position: absolute;"} + params.close_css.clone();
    let close_button = dom_element! {
        <button
            css={close_css}
            type="button"
            title={&params.close_label}
            aria-label={&params.close_label}
            on_click={bind!(close, |_: ClickEvent| close())}
        >
            "×"
        </button>
    };
    let last = close_button.get_ref();

    // The close button comes last, after the content, so Shift+Tab from the first element of
    // the content goes round to it; on the screen it is in the corner
    let window_css = css! {"
        position: relative;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        outline: none;
    "} + params.window_css.clone();
    let body_css = css! {"
        flex: 1 1 auto;
        min-height: 0;
        overflow-y: auto;
        overscroll-behavior: contain;
    "} + params.body_css.clone();
    let beside = Beside::default();
    let on_window_pointer_down = bind!(beside, |_: PointerEvent| beside.in_window());
    let on_window_click = bind!(beside, |_: ClickEvent| beside.in_window());
    // The body scrolls; the rest of the window doesn't, nor the page under the layer, see the
    // layer
    let on_body_wheel = |event: WheelEvent| event.stop_propagation();
    let window = dom_element! {
        <div
            css={window_css}
            role="dialog"
            aria-modal="true"
            aria-labelledby={&title_id}
            tabindex="-1"
            autofocus="autofocus"
            on_pointer_down={on_window_pointer_down}
            on_click={on_window_click}
            {..w.clone()}
        >
            <h2 css={params.title_css.clone()} id={&title_id}>{title}</h2>
            <div css={body_css} on_wheel={on_body_wheel}>{content()}</div>
            {close_button}
        </div>
    };
    let first = window.get_ref();
    let opened = Opened::new(number);

    // Two stops of the keyboard around the window catch the focus leaving it and send it
    // round: past the end back to the window, before the start to the close button
    let to_first = move || focus(&first);
    let to_last = move || focus(&last);

    // Escape anywhere on the page closes the window on top, also when the focus got lost. An
    // element handling Escape itself stops it before it gets to the page. A window opened over
    // another listens since after it, so the one under hears the Escape first, while it isn't
    // the top one yet
    let on_key_down = bind!(close, |event: KeyDownEvent| {
        if event.key == "Escape" && is_top(number) {
            close();
            return true;
        }
        false
    });
    let on_overlay_pointer_down = bind!(beside, |_: PointerEvent| beside.pressed());
    let on_overlay_click = bind!(beside, close, |_: ClickEvent| if beside.clicked() {
        close()
    });
    // A wheel over the layer would scroll what is under it, which isn't the page only: any
    // scrolling ancestor of the component
    let on_overlay_wheel = |event: WheelEvent| event.prevent_default();

    let overlay = dom! {
        <div
            css={params.overlay_css.clone()}
            on_pointer_down={on_overlay_pointer_down}
            on_click={on_overlay_click}
            on_wheel={on_overlay_wheel}
            hook_key_down={on_key_down}
        >
            <span tabindex="0" on_focus={to_last} />
            {window}
            <span tabindex="0" on_focus={to_first} />
        </div>
    };
    // The window is open, keeping the page from scrolling, as long as the layer is there. Then the
    // focus, gone with it, goes back
    overlay.append_drop_resource(DropResource::from_struct(opened));
    overlay.append_drop_resource(focus_back);
    overlay
}

fn focus(element: &DomElementRef) {
    js! { #element.focus() };
}

thread_local! {
    static NEXT_NUMBER: Cell<u64> = const { Cell::new(0) };
}

/// The number of a window, different in every window opened: in the id of its title, which names
/// it for a screen reader, and in [`OPEN`].
fn next_number() -> u64 {
    NEXT_NUMBER.with(|next| next.replace(next.get() + 1))
}

/// Whether a click of the layer is beside the window: pressed beside it as well. A press in the
/// window let go beside it, f. ex. selecting text in a field, clicks the layer too, as the element
/// both are in, and a click in the window comes up to the layer as any other event does.
#[derive(Clone, Default)]
struct Beside {
    /// The press or the click went through the window on its way up to the layer.
    in_window: Rc<Cell<bool>>,
    /// The last press was beside the window.
    pressed: Rc<Cell<bool>>,
}

impl Beside {
    /// A press or a click in the window, before it gets to the layer.
    fn in_window(&self) {
        self.in_window.set(true);
    }

    /// A press got to the layer.
    fn pressed(&self) {
        self.pressed.set(!self.in_window.take());
    }

    /// A click got to the layer: whether it closes the window.
    fn clicked(&self) -> bool {
        let let_go_beside = !self.in_window.take();
        let pressed_beside = self.pressed.take();
        let_go_beside && pressed_beside
    }
}

/// The style of the page's root element that keeps it from scrolling under the windows.
const OVERFLOW: &str = "overflow";
/// Keeps the room of the page's scrollbar, so the page doesn't move when the scrollbar goes.
const SCROLLBAR_GUTTER: &str = "scrollbar-gutter";

thread_local! {
    /// The windows open, the last one on top, and the page's own style the first of them
    /// replaced, given back when the last one closes.
    static OPEN: RefCell<Open> = const {
        RefCell::new(Open {
            windows: Vec::new(),
            replaced: Vec::new(),
        })
    };
}

struct Open {
    /// The number of each window.
    windows: Vec<u64>,
    replaced: Vec<(&'static str, JsJson)>,
}

/// A window open, on top of those opened before it, till dropped. The page doesn't scroll while
/// any is open.
struct Opened(u64);

impl Opened {
    fn new(number: u64) -> Self {
        OPEN.with_borrow_mut(|open| {
            if open.windows.is_empty() {
                open.replaced = lock_page();
            }
            open.windows.push(number);
        });
        Self(number)
    }
}

impl Drop for Opened {
    fn drop(&mut self) {
        OPEN.with_borrow_mut(|open| {
            open.windows.retain(|number| *number != self.0);
            if open.windows.is_empty() {
                unlock_page(std::mem::take(&mut open.replaced));
            }
        });
    }
}

/// Whether the window of `number` is on top.
fn is_top(number: u64) -> bool {
    OPEN.with_borrow(|open| open.windows.last() == Some(&number))
}

/// Keeps the page from scrolling, keeping the room of its scrollbar. Gives back the style of the
/// page's root element it replaced.
fn lock_page() -> Vec<(&'static str, JsJson)> {
    let replaced = [OVERFLOW, SCROLLBAR_GUTTER]
        .map(|name| (name, page_style(name)))
        .into();
    if has_scrollbar() {
        set_page_style(SCROLLBAR_GUTTER, "stable".into());
    }
    set_page_style(OVERFLOW, "hidden".into());
    replaced
}

fn unlock_page(replaced: Vec<(&'static str, JsJson)>) {
    for (name, value) in replaced {
        set_page_style(name, value);
    }
}

/// The page's scrollbar takes room of the window, which a scrollbar over the page doesn't.
fn has_scrollbar() -> bool {
    let inner = js! { window.innerWidth };
    let client = js! { document.documentElement.clientWidth };
    match (inner, client) {
        (JsJson::Number(inner), JsJson::Number(client)) => inner.0 > client.0,
        _ => false,
    }
}

/// The style of the page's root element set in its `style` attribute.
fn page_style(name: &str) -> JsJson {
    js! { document.documentElement.style.getPropertyValue(name) }
}

/// Sets the style of the page's root element, or takes it away with an empty string.
fn set_page_style(name: &str, value: JsJson) {
    js! { document.documentElement.style.setProperty(name, value) };
}
