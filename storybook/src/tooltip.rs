use vertigo::{DomNode, Value, css, dom};
use vertigo_forms::{Tooltip, TooltipParams};

pub fn tooltip() -> DomNode {
    let row = css! {"
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin: 1em 0;
    "};
    let fed = Value::new(0);
    let feed = {
        let fed = fed.clone();
        move |_| fed.change(|fed| *fed += 1)
    };
    let fed_times = fed.map(|fed| format!("Fed {fed} times"));

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
            <p>
                "A button with a hint: "
                <Tooltip
                    element={dom! { <button on_click={feed}>"Feed the cat"</button> }}
                    content={dom! { <span>"The cat had its breakfast an hour ago"</span> }}
                    params={TooltipParams { control: true, ..TooltipParams::default() }}
                />
                " "
                <span>{fed_times}</span>
                ". Point at it; a click feeds the cat."
            </p>
        </div>
    }
}
