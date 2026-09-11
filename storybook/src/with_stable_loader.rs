use vertigo::{DomNode, Resource, Value, bind, bind_rc, css, dom};
use vertigo_forms::{Input, WithLoader, WithStableLoader};

pub fn with_stable_loader() -> DomNode {
    let resource = Value::new(Resource::Loading);

    // A field the viewer can type into, so it is visible which of the two loaders keeps
    // the state the user is working in and which one throws it away.
    let render = bind_rc!(|value: String| {
        let draft = Value::default();

        dom! {
            <div>
                <p>"Resource ready: " {value}</p>
                <p>"Type here: " <Input value={draft} /></p>
            </div>
        }
    });

    let set_loading = bind!(resource, |_| resource.set(Resource::Loading));
    let set_ready = bind!(resource, |_| resource
        .set(Resource::Ready("Updated value".to_string())));
    let set_error = bind!(resource, |_| resource
        .set(Resource::Error("Deliberate error".to_string())));

    let css_columns = css! {"
        display: flex;
        gap: 20px;
    "};

    let css_column = css! {"
        flex: 1;
        border: 1px solid #ccc;
        padding: 10px;
    "};

    dom! {
        <div>
            <p>
                <button on_click={set_loading}>"Set loading"</button>
                <button on_click={set_ready}>"Set ready"</button>
                <button on_click={set_error}>"Set error"</button>
            </p>

            <p>
                "Both loaders watch the same resource. Click \"Set ready\", type into the field,
                then click \"Set ready\" again (or \"Set loading\" / \"Set error\"): the stable
                loader keeps what you typed, the plain one rebuilds and loses it."
            </p>

            <div css={css_columns}>
                <div css={&css_column}>
                    <h3>"WithStableLoader"</h3>
                    <WithStableLoader resource={resource.to_computed()} render={&render} />
                </div>
                <div css={css_column}>
                    <h3>"WithLoader"</h3>
                    <WithLoader resource={resource.to_computed()} {render} />
                </div>
            </div>
        </div>
    }
}
