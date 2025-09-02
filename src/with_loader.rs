use std::rc::Rc;
use vertigo::{Computed, DomNode, Resource, component, dom};

use crate::Spinner;

/// Wraps [Resource\<T\>](https://docs.rs/vertigo/latest/vertigo/enum.Resource.html)
/// and shows [Spinner], error or renders value based on Resource variant.
///
/// See also [with_loader].
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
/// See also [WithLoader].
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
