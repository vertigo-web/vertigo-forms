use vertigo::{AttrGroup, Computed, Css, DomNode, bind, component, css, dom};

#[derive(Clone, Default)]
pub struct PopupParams {
    pub css: Css,
}

fn popup_css() -> Css {
    css! {"
        visibility: hidden;
        position: absolute;
        z-index: 1;
    "}
}

fn operator_css() -> Css {
    css! {"
        position: relative;
        display: inline-block;
    "}
}

#[component]
pub fn Popup(
    visible: Computed<bool>,
    content: DomNode,
    params: PopupParams,
    /// Additional attributes for the container
    c: AttrGroup,
    /// Additional attributes for the popup
    p: AttrGroup,
) {
    let popup_css = popup_css();

    let container_css = bind!(
        popup_css,
        params,
        visible.map(move |enabled| {
            let base_css = operator_css();

            if enabled {
                base_css + css! {"[popup_css] { visibility: visible; }"}
            } else {
                base_css
            }
        })
    );

    dom! {
        <div css={container_css} {..c}>
            <div css={popup_css + params.css} {..p}>
                {content}
            </div>
        </div>
    }
}

#[component]
pub fn PopupOnHover(
    element: DomNode,
    content: DomNode,
    params: PopupParams,
    /// Additional attributes for the container
    c: AttrGroup,
    /// Additional attributes for the popup
    p: AttrGroup,
) {
    let popup_css = popup_css();

    let operator_css = operator_css() + css! {":hover [popup_css] { visibility: visible; }"};

    dom! {
        <div css={operator_css} {..c}>
            {element}
            <div css={popup_css + params.css} {..p}>
                {content}
            </div>
        </div>
    }
}
