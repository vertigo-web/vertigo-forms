use vertigo::{DomNode, Resource, Value, bind, bind_rc, dom};
use vertigo_forms::WithLoader;

pub fn with_loader() -> DomNode {
    let resource = Value::new(Resource::Ready("Initial value".to_string()));

    let render = bind_rc!(|value: _| {
        dom! {
            <main>"Resource ready: " {value}</main>
        }
    });

    let set_loading = bind!(resource, |_| resource.set(Resource::Loading));
    let set_ready = bind!(resource, |_| resource
        .set(Resource::Ready("Updated value".to_string())));
    let set_error = bind!(resource, |_| resource
        .set(Resource::Error("Deliberate error".to_string())));

    dom! {
        <div>
            <p>
                <button on_click={set_loading}>"Set loading"</button>
                <button on_click={set_ready}>"Set ready"</button>
                <button on_click={set_error}>"Set error"</button>
            </p>

            <WithLoader resource={resource.to_computed()} {render} />
        </div>
    }
}
