use vertigo::{bind, css, dom, Computed, Css, DomNode, Value};

use super::multi_select::MultiSelect;

#[derive(Clone, Default)]
pub struct MultiDropDownParams {
    pub drop_down_content_css: Css,
}

#[derive(Clone)]
pub struct MultiDropDown<T: Clone + 'static> {
    pub value: Value<Vec<T>>,
    pub options: Computed<Vec<T>>,
    pub params: MultiDropDownParams,
}

impl<T> MultiDropDown<T>
where
    T: Clone + From<String> + PartialEq + ToString + 'static
{
    pub fn mount(self) -> DomNode {
        let opened = Value::new(false);
        let button_label = opened.render_value(|opened| {
            if opened {
                dom! { <span>"^"</span> }
            } else {
                dom! { <span>"V"</span> }
            }
        });
        let content_css = css! {"
            position: absolute;
            z-index: 1;
        "};
        let drop_down_content_css = content_css.extend(self.params.drop_down_content_css);

        let content = opened.render_value(move |opened|
            if opened {
                dom! {
                    <div css={drop_down_content_css.clone()}>
                        <MultiSelect
                            value={&self.value}
                            options={&self.options}
                        />
                    </div>
                }
            } else {
                dom! { <div/> }
            }
        );

        let on_click = bind!(opened, || opened.change(|o| *o = !*o));

        let base_css = css! {"
            position: relative;
            display: inline-block;
        "};

        dom! {
            <div>
                <button {on_click}>{button_label}</button>
                <div css={base_css}>{content}</div>
            </div>
        }
    }
}
