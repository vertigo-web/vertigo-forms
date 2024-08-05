use vertigo::{css, dom, DomNode, Value};
use vertigo_forms::{Popup, PopupOnHover, PopupParams, Switch};

pub fn popup() -> DomNode {
    let visible = Value::new(false);

    let params = PopupParams {
        css: css! {"
            border-style: solid;
            border-width: 2px;
            border-radius: 10px;
            padding: 0px 10px;
            background-color: white;
        "},
    };

    let content = dom! {
        <p>"Content in the popup"</p>
    };

    dom! {
        <p>
            "Popup with trigger: "
            <Switch
                value={&visible}
                params={}
            />
            <Popup {visible} {content} params={params.clone()} />
        </p>
        <p>
            <PopupOnHover
                element={dom! { <p>"Popup on hover"</p> }}
                content={dom! { <p>"Hover popup content"</p> }}
                {params}
            />
        </p>
    }
}
