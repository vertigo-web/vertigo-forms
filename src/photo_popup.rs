use std::{cell::Cell, rc::Rc};

use vertigo::{
    AttrGroup, ClickEvent, Computed, Css, DomNode, KeyDownEvent, PointerEvent, PointerType, Value,
    WheelEvent, bind, component, css, dom,
};

/// A photo of [`PhotoGallery`] and [`PhotoPopup`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Photo {
    /// The picture, also where the thumbnail of [`PhotoGallery`] links to.
    pub url: String,
    pub alt: String,
}

#[derive(Clone)]
pub struct PhotoPopupParams {
    /// The layer over the whole page, with the photo in the middle.
    pub overlay_css: Css,
    /// The photo, fitted into the window.
    pub photo_css: Css,
    /// How the close, previous and next buttons look. Where they are is up to the component.
    pub button_css: Css,
    /// "2 / 5", shown when there's more than one photo.
    pub counter_css: Css,
    /// Titles of the buttons, shown on hover and read out by screen readers.
    pub close_label: String,
    pub previous_label: String,
    pub next_label: String,
}

impl Default for PhotoPopupParams {
    fn default() -> Self {
        Self {
            overlay_css: css! {"
                position: fixed;
                inset: 0;
                z-index: 1000;
                display: flex;
                align-items: center;
                justify-content: center;
                background-color: rgba(0, 0, 0, 0.85);
            "},
            photo_css: css! {"
                max-width: 92vw;
                max-height: 88vh;
                object-fit: contain;
            "},
            button_css: css! {"
                width: 48px;
                height: 48px;
                padding: 0;
                border: none;
                border-radius: 50%;
                background-color: rgba(0, 0, 0, 0.4);
                color: white;
                font-size: 32px;
                line-height: 1;
                cursor: pointer;

                :hover {
                    background-color: rgba(0, 0, 0, 0.7);
                };
            "},
            counter_css: css! {"
                color: white;
                font-size: 14px;
            "},
            close_label: "Close".to_string(),
            previous_label: "Previous".to_string(),
            next_label: "Next".to_string(),
        }
    }
}

/// One of `photos` over the whole page, the one at the index in `shown`. Nothing, when `shown`
/// is `None`.
///
/// Escape, the close button or a click beside the photo set `shown` back to `None`. With more
/// photos than one, the arrow keys, buttons and a swipe of a finger go through them, from the
/// last back to the first. Two fingers still zoom the page.
///
/// The layer has `position: fixed`, so no ancestor of the component may have a `transform` or
/// a `filter`, which would make it fixed to that ancestor instead of the window.
///
/// Example:
/// ```
/// use vertigo::{Value, bind, dom};
/// use vertigo_forms::{Photo, PhotoPopup};
///
/// let photos = vec![Photo { url: "/photo.jpg".to_string(), alt: "A photo".to_string() }];
/// let shown = Value::new(None);
/// let show = bind!(shown, |_| shown.set(Some(0)));
/// dom! {
///     <button on_click={show}>"Show"</button>
///     <PhotoPopup {photos} {shown} params={} />
/// };
/// ```
#[component]
pub fn PhotoPopup(photos: Vec<Photo>, shown: Value<Option<usize>>, params: PhotoPopupParams) {
    let photos = Rc::new(photos);
    let count = photos.len();
    let current = shown.map(move |shown| shown.filter(|index| *index < count));

    current
        .map(|current| current.is_some())
        .render_value_option(move |open| {
            open.then(|| render_overlay(&photos, &shown, &current, &params))
        })
}

fn render_overlay(
    photos: &Rc<Vec<Photo>>,
    shown: &Value<Option<usize>>,
    current: &Computed<Option<usize>>,
    params: &PhotoPopupParams,
) -> DomNode {
    let count = photos.len();
    let photo = {
        let photos = photos.clone();
        current.map(move |current| {
            current
                .and_then(|index| photos.get(index))
                .cloned()
                .unwrap_or_default()
        })
    };

    let close = bind!(shown, || shown.set(None));
    let previous = bind!(shown, || shown.change(|shown| step(
        shown,
        count,
        count - 1
    )));
    let next = bind!(shown, || shown.change(|shown| step(shown, count, 1)));

    let on_key_down = bind!(close, previous, next, |event: KeyDownEvent| {
        match event.key.as_str() {
            "Escape" => close(),
            "ArrowLeft" if count > 1 => previous(),
            "ArrowRight" if count > 1 => next(),
            _ => return false,
        }
        true
    });

    let button = |label: &str, symbol: &str, place: Css, action: Rc<dyn Fn()>| {
        let on_click = move |event: ClickEvent| {
            event.stop_propagation();
            action();
        };
        dom! {
            <button
                css={place + params.button_css.clone()}
                type="button"
                title={label}
                aria-label={label}
                on_click={on_click}
            >
                {symbol}
            </button>
        }
    };

    let close_button = button(
        &params.close_label,
        "×",
        css! {"position: absolute; top: 12px; right: 12px;"},
        Rc::new(close.clone()),
    );

    let browsing = count > 1;
    let side = css! {"position: absolute; top: 50%; transform: translateY(-50%);"};
    let previous_button = browsing.then(|| {
        button(
            &params.previous_label,
            "‹",
            side.clone() + css! {"left: 12px;"},
            Rc::new(previous.clone()),
        )
    });
    let next_button = browsing.then(|| {
        button(
            &params.next_label,
            "›",
            side + css! {"right: 12px;"},
            Rc::new(next.clone()),
        )
    });
    let counter = browsing.then(|| {
        let text = current.map(move |current| {
            current
                .map(|index| format!("{} / {count}", index + 1))
                .unwrap_or_default()
        });
        let counter_css = css! {"
            position: absolute;
            bottom: 12px;
            left: 50%;
            transform: translateX(-50%);
        "} + params.counter_css.clone();
        dom! { <span css={counter_css}>{text}</span> }
    });

    let swipe = Swipe::default();
    let on_pointer_down = bind!(swipe, |event: PointerEvent| if browsing {
        swipe.down(&event)
    });
    let on_pointer_move = bind!(swipe, |event: PointerEvent| swipe.moved(&event));
    let on_pointer_up = bind!(swipe, previous, next, |event: PointerEvent| {
        match swipe.up(&event) {
            Some(Direction::Previous) => previous(),
            Some(Direction::Next) => next(),
            None => {}
        }
    });
    let on_pointer_cancel = bind!(swipe, |_: PointerEvent| swipe.cancel());
    // A pen ends a swipe with a click, a finger doesn't
    let on_click = bind!(swipe, |_: ClickEvent| if !swipe.swiped.get() {
        close()
    });
    let photo_style = swipe
        .shift
        .to_computed()
        .map(|shift| (shift != 0.0).then(|| format!("transform: translateX({shift}px);")));

    // The browser handles two fingers (zoom), the component one (swipe)
    let overlay_css =
        css! {"outline: none; touch-action: pinch-zoom;"} + params.overlay_css.clone();
    let on_wheel = |event: WheelEvent| event.prevent_default();
    let on_photo_click = |event: ClickEvent| event.stop_propagation();

    // The layer takes the focus, so Tab goes through the buttons of the popup
    dom! {
        <div
            css={overlay_css}
            role="dialog"
            aria-modal="true"
            tabindex="-1"
            autofocus="autofocus"
            on_click={on_click}
            on_wheel={on_wheel}
            on_pointer_down={on_pointer_down}
            on_pointer_move={on_pointer_move}
            on_pointer_up={on_pointer_up}
            on_pointer_cancel={on_pointer_cancel}
            hook_key_down={on_key_down}
        >
            <img
                css={params.photo_css.clone()}
                style={photo_style}
                src={photo.map(|photo| photo.url)}
                alt={photo.map(|photo| photo.alt)}
                draggable="false"
                on_click={on_photo_click}
            />
            {close_button}
            {..previous_button}
            {..next_button}
            {..counter}
        </div>
    }
}

/// Moves the shown photo `by` forward, around to the first after the last.
fn step(shown: &mut Option<usize>, count: usize, by: usize) {
    if let Some(index) = shown {
        *index = (*index + by) % count;
    }
}

/// How far a finger has to go across to swipe, in CSS pixels.
const SWIPE_DISTANCE: f64 = 50.0;

#[derive(Debug, PartialEq)]
enum Direction {
    Previous,
    Next,
}

/// A finger (or a pen) going across the popup to another photo, which follows it meanwhile.
#[derive(Clone, Default)]
struct Swipe {
    /// Id of the pointer and where it touched the popup.
    start: Rc<Cell<Option<(i32, f64, f64)>>>,
    /// How far the photo is moved after the pointer, in CSS pixels.
    shift: Value<f64>,
    /// Whether the last pointer swiped, so the click it may end with doesn't close the popup.
    swiped: Rc<Cell<bool>>,
}

impl Swipe {
    /// Only the first finger swipes, a mouse has the buttons and the keys. A second finger
    /// makes it a pinch, and the browser zooms.
    fn down(&self, event: &PointerEvent) {
        self.swiped.set(false);
        let starts = event.pointer_type != PointerType::Mouse && event.is_primary;
        self.start
            .set(starts.then_some((event.pointer_id, event.pointer_x, event.pointer_y)));
        self.shift.set(0.0);
    }

    fn moved(&self, event: &PointerEvent) {
        if let Some((id, x, _)) = self.start.get()
            && id == event.pointer_id
        {
            self.shift.set(event.pointer_x - x);
        }
    }

    /// Where the pointer swiped to, if it went far enough across rather than up or down.
    fn up(&self, event: &PointerEvent) -> Option<Direction> {
        let (id, x, y) = self.start.take()?;
        self.shift.set(0.0);
        let (across, down) = (event.pointer_x - x, event.pointer_y - y);
        if id != event.pointer_id || across.abs() < SWIPE_DISTANCE || across.abs() <= down.abs() {
            return None;
        }
        self.swiped.set(true);
        Some(if across < 0.0 {
            Direction::Next
        } else {
            Direction::Previous
        })
    }

    /// The browser took the pointer over, f. ex. to zoom.
    fn cancel(&self) {
        self.start.set(None);
        self.shift.set(0.0);
    }
}

#[derive(Clone)]
pub struct PhotoGalleryParams {
    /// The container of the thumbnails.
    pub css: Css,
    /// Each thumbnail.
    pub thumbnail_css: Css,
    pub popup: PhotoPopupParams,
}

impl Default for PhotoGalleryParams {
    fn default() -> Self {
        Self {
            css: css! {"
                display: flex;
                flex-wrap: wrap;
                gap: 8px;
            "},
            thumbnail_css: css! {"
                height: 150px;
                width: auto;
            "},
            popup: PhotoPopupParams::default(),
        }
    }
}

/// Thumbnails of `photos`, each opening its photo in a [`PhotoPopup`].
///
/// A thumbnail is a link to the photo, opened in a new tab, so it works the same before the app
/// starts, for search engines, and with Ctrl/⌘/Shift or the middle mouse button.
///
/// Example:
/// ```
/// use vertigo::dom;
/// use vertigo_forms::{Photo, PhotoGallery};
///
/// let photos = ["/first.jpg", "/second.jpg"]
///     .map(|url| Photo { url: url.to_string(), alt: "Our work".to_string() })
///     .to_vec();
/// dom! {
///     <PhotoGallery {photos} params={} t:loading="lazy" />
/// };
/// ```
#[component]
pub fn PhotoGallery(
    photos: Vec<Photo>,
    params: PhotoGalleryParams,
    /// Additional attributes for each thumbnail `<img>`
    t: AttrGroup,
) {
    let shown = Value::default();
    let t = t.clone();

    let thumbnails = photos.iter().enumerate().map(|(index, photo)| {
        let on_click = bind!(shown, |event: ClickEvent| {
            if !(event.ctrl_key || event.meta_key || event.shift_key || event.alt_key) {
                event.prevent_default();
                shown.set(Some(index));
            }
        });
        dom! {
            <a
                css={css! {"cursor: zoom-in;"}}
                href={&photo.url}
                target="_blank"
                rel="noopener"
                on_click={on_click}
            >
                <img
                    css={params.thumbnail_css.clone()}
                    src={&photo.url}
                    alt={&photo.alt}
                    {..t.clone()}
                />
            </a>
        }
    });

    dom! {
        <div css={params.css}>
            {..thumbnails}
            <PhotoPopup {photos} {shown} params={params.popup} />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use vertigo::{PointerEvent, PointerType, transaction};

    use super::{Direction, Swipe, step};

    fn finger(pointer_x: f64, pointer_y: f64) -> PointerEvent {
        // `PointerEvent` has private fields, so no `..Default::default()`
        let mut event = PointerEvent::default();
        event.pointer_id = 1;
        event.pointer_type = PointerType::Touch;
        event.is_primary = true;
        event.pointer_x = pointer_x;
        event.pointer_y = pointer_y;
        event
    }

    fn shift(swipe: &Swipe) -> f64 {
        transaction(|ctx| swipe.shift.get(ctx))
    }

    #[test]
    fn swipe_left_goes_to_next_and_right_to_previous() {
        let swipe = Swipe::default();
        swipe.down(&finger(300.0, 200.0));
        swipe.moved(&finger(220.0, 210.0));
        assert_eq!(shift(&swipe), -80.0);
        assert_eq!(swipe.up(&finger(200.0, 210.0)), Some(Direction::Next));
        assert_eq!(shift(&swipe), 0.0);
        assert!(swipe.swiped.get());

        swipe.down(&finger(100.0, 200.0));
        assert!(!swipe.swiped.get());
        assert_eq!(swipe.up(&finger(200.0, 170.0)), Some(Direction::Previous));
    }

    #[test]
    fn short_or_steep_move_is_no_swipe() {
        let swipe = Swipe::default();
        swipe.down(&finger(300.0, 200.0));
        assert_eq!(swipe.up(&finger(260.0, 200.0)), None);
        swipe.down(&finger(300.0, 200.0));
        assert_eq!(swipe.up(&finger(200.0, 320.0)), None);
        assert!(!swipe.swiped.get());
    }

    #[test]
    fn mouse_second_finger_and_cancel_are_no_swipe() {
        let swipe = Swipe::default();
        let mouse = |x| {
            let mut event = finger(x, 200.0);
            event.pointer_type = PointerType::Mouse;
            event
        };
        swipe.down(&mouse(300.0));
        swipe.moved(&mouse(100.0));
        assert_eq!(shift(&swipe), 0.0);
        assert_eq!(swipe.up(&mouse(100.0)), None);

        swipe.down(&finger(300.0, 200.0));
        let mut second = finger(400.0, 200.0);
        second.pointer_id = 2;
        second.is_primary = false;
        swipe.down(&second);
        assert_eq!(swipe.up(&finger(100.0, 200.0)), None);

        swipe.down(&finger(300.0, 200.0));
        swipe.moved(&finger(200.0, 200.0));
        swipe.cancel();
        assert_eq!(shift(&swipe), 0.0);
        assert_eq!(swipe.up(&finger(100.0, 200.0)), None);
    }

    #[test]
    fn step_goes_around() {
        let mut shown = Some(2);
        step(&mut shown, 3, 1);
        assert_eq!(shown, Some(0));
        step(&mut shown, 3, 2);
        assert_eq!(shown, Some(2));
        step(&mut shown, 3, 2);
        assert_eq!(shown, Some(1));

        let mut closed = None;
        step(&mut closed, 3, 1);
        assert_eq!(closed, None);
    }
}
