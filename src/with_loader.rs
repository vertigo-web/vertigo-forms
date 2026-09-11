use std::{cell::Cell, rc::Rc};
use vertigo::{Computed, DomNode, Resource, component, dom, transaction};

use crate::Spinner;

/// Wraps [Resource\<T\>](https://docs.rs/vertigo/latest/vertigo/enum.Resource.html)
/// and shows [Spinner], error or renders value based on Resource variant.
///
/// See also [with_loader] and [WithStableLoader].
#[component]
pub fn WithLoader<T: Clone + PartialEq + 'static>(
    resource: Computed<Resource<T>>,
    render: Rc<dyn Fn(T) -> DomNode>,
) {
    with_loader(resource, render)
}

/// Wraps [Resource\<T\>](https://docs.rs/vertigo/latest/vertigo/enum.Resource.html)
/// and shows [Spinner], error or renders value based on Resource variant.
///
/// See also [WithLoader] and [with_stable_loader].
pub fn with_loader<T: Clone + PartialEq + 'static>(
    resource: Computed<Resource<T>>,
    render: Rc<dyn Fn(T) -> DomNode>,
) -> DomNode {
    resource.render_value(move |res| match res {
        Resource::Loading => dom! {
            <Spinner />
        },
        Resource::Ready(value) => render(value),
        Resource::Error(err) => dom! {
            <main>{err}</main>
        },
    })
}

/// Which branch of a [Resource\<T\>](https://docs.rs/vertigo/latest/vertigo/enum.Resource.html)
/// is showing, without the value itself.
///
/// Keeping the value out is the whole point: this is what the stable loader below subscribes
/// to, so a re-fetch that lands on the same branch changes nothing.
#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready,
    Error(String),
}

/// Like [with_loader], but `render` runs once, when the resource first becomes ready,
/// instead of on every value it takes afterwards.
///
/// See also [WithStableLoader].
///
/// [with_loader] subscribes to the whole `Resource`, so every re-fetch rebuilds the subtree
/// it renders. That is right for a read-only view and wrong for an edit form: re-fetching
/// after a save would throw away the fields the user is working in, the active tab and any
/// "saved" label. So the subscription here is on the `Loading` / `Ready` / `Error` branch
/// alone, and `render` gets the value as it stands when the resource becomes ready.
///
/// Ready is sticky: once the content is up, a later `Loading` or `Error` leaves it alone.
/// A form the user is halfway through is worth more than the error message from a background
/// refresh that failed - the save itself is expected to report its own outcome.
///
/// Anything that *should* follow later updates - a page heading, say - subscribes to the
/// resource itself, alongside this.
pub fn with_stable_loader<T: Clone + PartialEq + 'static>(
    resource: Computed<Resource<T>>,
    render: Rc<dyn Fn(T) -> DomNode>,
) -> DomNode {
    // Only ever flipped on, so it does not matter how often the closure below runs.
    let was_ready = Rc::new(Cell::new(false));

    let state = resource.map({
        let was_ready = was_ready.clone();

        move |res| match res {
            Resource::Ready(_) => {
                was_ready.set(true);
                LoadState::Ready
            }
            Resource::Loading if was_ready.get() => LoadState::Ready,
            Resource::Loading => LoadState::Loading,
            Resource::Error(_) if was_ready.get() => LoadState::Ready,
            Resource::Error(err) => LoadState::Error(err),
        }
    });

    state.render_value(move |state| match state {
        LoadState::Loading => dom! { <Spinner /> },
        LoadState::Error(err) => dom! { <main>{err}</main> },
        LoadState::Ready => match transaction(|ctx| resource.get(ctx)) {
            Resource::Ready(value) => render(value),
            // Unreachable: `state` only *enters* Ready on the change that made the
            // resource ready, and a sticky Ready afterwards is not a change, so this
            // callback does not run again.
            _ => dom! { <Spinner /> },
        },
    })
}

/// Like [WithLoader], but `render` runs once, when the resource first becomes ready,
/// instead of on every value it takes afterwards.
///
/// See [with_stable_loader] for the details.
#[component]
pub fn WithStableLoader<T: Clone + PartialEq + 'static>(
    resource: Computed<Resource<T>>,
    render: Rc<dyn Fn(T) -> DomNode>,
) {
    with_stable_loader(resource, render)
}
