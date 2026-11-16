use vertigo::{DomNode, css, dom};
use vertigo_forms::Tooltip;

pub fn tooltip() -> DomNode {
    let row = css! {"
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin: 1em 0;
    "};

    dom! {
        <div>
            <p>
                "A word with a hint: "
                <Tooltip
                    element={dom! { <b>"cat"</b> }}
                    content={dom! { <span>"Clearly not a dog"</span> }}
                    params={}
                />
                ". Point at it, tap it, or reach it with Tab and press Enter."
            </p>
            <div css={row}>
                <span>"A sign at the right edge:"</span>
                <Tooltip
                    element={dom! { <span>"★ 4.5"</span> }}
                    content={dom! {
                        <span>"4.5 out of 5 stars from 120 reviews, most of them praising the friendly service"</span>
                    }}
                    params={}
                />
            </div>
        </div>
    }
}
