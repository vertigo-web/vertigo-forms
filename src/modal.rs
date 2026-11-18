use std::{cell::Cell, rc::Rc};

use vertigo::{
    AttrGroup, ClickEvent, Css, DomElementRef, DomNode, KeyDownEvent, Value, WheelEvent, bind,
    component, css, dom, dom_element, js,
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
/// Escape, the close button or a click beside the window set `open` to `false`. While the window
/// is open, the page under it doesn't scroll, and the keyboard stays in the window: Tab past its
/// last element comes back to the first, and Shift+Tab past the first goes to the last, the close
/// button. A screen reader reads it as a dialog named by its title.
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
    let title_id = next_title_id();

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
    let on_window_click = |event: ClickEvent| event.stop_propagation();
    // The body scrolls; the page under the layer doesn't, see the layer
    let on_window_wheel = |event: WheelEvent| event.stop_propagation();
    let window = dom_element! {
        <div
            css={window_css}
            role="dialog"
            aria-modal="true"
            aria-labelledby={&title_id}
            tabindex="-1"
            autofocus="autofocus"
            on_click={on_window_click}
            on_wheel={on_window_wheel}
            {..w.clone()}
        >
            <h2 css={params.title_css.clone()} id={&title_id}>{title}</h2>
            <div css={body_css}>{content()}</div>
            {close_button}
        </div>
    };
    let first = window.get_ref();

    // Two stops of the keyboard around the window catch the focus leaving it and send it
    // round: past the end back to the window, before the start to the close button
    let to_first = move || focus(&first);
    let to_last = move || focus(&last);

    let on_key_down = bind!(close, |event: KeyDownEvent| {
        if event.key == "Escape" {
            close();
            return true;
        }
        false
    });
    let on_overlay_click = bind!(close, |_: ClickEvent| close());
    // A wheel over the layer would scroll what is under it, which isn't the page only: any
    // scrolling ancestor of the component. The lock of the page lives as long as the layer, as
    // this callback of it does
    let lock = ScrollLock::new();
    let on_overlay_wheel = move |event: WheelEvent| {
        let _ = &lock;
        event.prevent_default();
    };

    dom! {
        <div
            css={params.overlay_css.clone()}
            on_click={on_overlay_click}
            on_wheel={on_overlay_wheel}
            hook_key_down={on_key_down}
        >
            <span tabindex="0" on_focus={to_last} />
            {window}
            <span tabindex="0" on_focus={to_first} />
        </div>
    }
}

fn focus(element: &DomElementRef) {
    js! { #element.focus() };
}

thread_local! {
    static NEXT_TITLE: Cell<u64> = const { Cell::new(0) };
}

/// An id for the title of a window, which names it for a screen reader, different in every
/// window opened.
fn next_title_id() -> String {
    NEXT_TITLE.with(|next| {
        let number = next.get();
        next.set(number + 1);
        format!("vertigo-forms-modal-title-{number}")
    })
}

/// Keeps the page from scrolling under an open window, till dropped.
struct ScrollLock;

impl ScrollLock {
    fn new() -> Self {
        js! { document.documentElement.style.setProperty("overflow", "hidden") };
        Self
    }
}

impl Drop for ScrollLock {
    fn drop(&mut self) {
        js! { document.documentElement.style.removeProperty("overflow") };
    }
}
