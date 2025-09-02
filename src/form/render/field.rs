use vertigo::{DomElement, Value, bind, component, css, dom, dom_element};

use crate::{DictSelect, DropImageFile, Select, SelectSearch, Switch, SwitchParams, input::Input};

use super::super::{DataField, DataFieldValue};

#[component]
pub fn Field<'a>(field: &'a DataField) {
    match &field.value {
        DataFieldValue::String(val) => {
            dom! { <Input input:name={&&field.key} value={val.value.clone()} /> }
        }
        DataFieldValue::TextArea(val) => {
            let on_input = bind!(val.value, |new_value: String| {
                value.set(new_value);
            });
            let el =
                dom_element! { <textarea name={&&field.key} {on_input}>{&val.value}</textarea> };
            if let Some(rows) = val.rows {
                el.add_attr("rows", rows);
            }
            if let Some(cols) = val.cols {
                el.add_attr("cols", cols);
            }
            el.into()
        }
        DataFieldValue::List(val) => {
            dom! { <Select value={val.value.clone()} options={&val.options} /> }
        }
        DataFieldValue::Bool(val) => {
            dom! {
                <Switch i:name={&&field.key} value={&val.value} params={SwitchParams::checkbox()} />
            }
        }
        DataFieldValue::Dict(val) => {
            dom! { <DictSelect value={val.value.clone()} options={&val.options} /> }
        }
        DataFieldValue::Multi(val) => {
            bind!(val.value, val.options, val.add_label,
                val.value.render_value(move |vals| {
                    let selects = DomElement::new("div");
                    let row_css = css! {"
                        display: flex;
                    "};

                    for (idx, tag_id) in vals.iter().enumerate() {
                        let on_click = bind!(value, |_| value.change(|list| {
                            list.remove(idx);
                        }));
                        selects.add_child(dom! {
                        <div css={&row_css}>
                            <SelectSearch value={tag_id.clone()} options={options.clone()} params={} />
                            <button {on_click}>"x"</button>
                        </div>
                    });
                    }

                    selects.add_child({
                        let on_click = bind!(value, |_| value
                            .change(|list| list.push(Value::new(0))));
                        dom! {
                            <button {on_click}>{&add_label}</button>
                        }
                    });

                    selects.into()
                })
            )
        }
        DataFieldValue::Image(val) => {
            let params = val.component_params.clone().unwrap_or_default();
            dom! { <DropImageFile
                item={val.value.clone()}
                original_link={val.original_link.clone()}
                {params}
            /> }
        }
        DataFieldValue::Custom(val) => (val.render)(),
        DataFieldValue::StaticCustom(render) => render(),
    }
}
