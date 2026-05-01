use std::rc::Rc;
use vertigo::{DomNode, Resource, Value, css, dom};
use vertigo_forms::{
    Input,
    resource_table::{
        ResourceTable, ResourceTableLabels, base_header_css, base_row_css, main_col_css,
        normal_col_css,
    },
};

#[derive(Clone, PartialEq, Default)]
struct MyModel {
    id: u32,
    name: String,
}

#[derive(Clone)]
struct MyModelForm {
    name: Value<String>,
}

pub fn resource_table_text_field() -> DomNode {
    let list_state = Value::new(vec![
        Value::new(Some(MyModel {
            id: 1,
            name: "Item 1".to_string(),
        })),
        Value::new(Some(MyModel {
            id: 2,
            name: "Item 2".to_string(),
        })),
    ]);

    let list = list_state.to_computed().map(|list| {
        let computed_list = list.iter().map(|v| v.to_computed()).collect::<Vec<_>>();
        Resource::Ready(Rc::new(computed_list))
    });

    let table = ResourceTable {
        list,
        title: "My Resources".to_string(),
        add_label: "Add Item".to_string(),
        table_css: css!(""),
        render_header: || {
            dom! {
                <div css={base_header_css() + css!("grid-template-columns: 50px 1fr 150px;")}>
                    <div>"ID"</div>
                    <div>"Name"</div>
                    <div>"Actions"</div>
                </div>
            }
        },
        render_filters: None,
        create_new_model: Rc::new(|| MyModel {
            id: 0,
            name: String::new(),
        }),
        create_form_model: |model| MyModelForm {
            name: Value::new(model.name.clone()),
        },
        update_model: |model, form, context| {
            let name = form.name.get(context);
            if name.is_empty() {
                return Err(vec!["Name cannot be empty".to_string()]);
            }
            let mut new_model = model.clone();
            new_model.name = name;
            Ok(new_model)
        },
        render_row_view: |model, create_buttons, _alert| {
            dom! {
                <div css={base_row_css() + css!("grid-template-columns: 50px 1fr 150px;")}>
                    <div css={normal_col_css()}>{model.id}</div>
                    <div css={main_col_css()}>{model.name.clone()}</div>
                    <div>{create_buttons()}</div>
                </div>
            }
        },
        render_row_form: |form, buttons| {
            dom! {
                <div css={base_row_css() + css!("grid-template-columns: 50px 1fr 150px;")}>
                    <div>"-"</div>
                    <div>
                        <Input value={form.name.clone()} />
                    </div>
                    <div>{buttons}</div>
                </div>
            }
        },
        on_create: Rc::new({
            let list_state = list_state.clone();
            move |new_model| {
                let list_state = list_state.clone();
                Box::pin(async move {
                    vertigo::transaction(|ctx| {
                        let mut m = new_model.clone();
                        let mut current = list_state.get(ctx);
                        m.id = current.len() as u32 + 10;
                        current.push(Value::new(Some(m)));
                        list_state.set(current);
                    });
                    None
                })
            }
        }),
        on_update: Rc::new({
            let list_state = list_state.clone();
            move |updated_model| {
                let list_state = list_state.clone();
                Box::pin(async move {
                    vertigo::transaction(|ctx| {
                        let current = list_state.get(ctx);
                        for item_val in current.iter() {
                            if let Some(ref m) = item_val.get(ctx)
                                && m.id == updated_model.id
                            {
                                item_val.set(Some(updated_model.clone()));
                                break;
                            }
                        }
                    });
                    None
                })
            }
        }),
        on_delete: Some(Rc::new({
            let list_state = list_state.clone();
            move |deleted_model| {
                let list_state = list_state.clone();
                Box::pin(async move {
                    vertigo::transaction(|ctx| {
                        let current = list_state.get(ctx);
                        for item_val in current.iter() {
                            if let Some(ref m) = item_val.get(ctx)
                                && m.id == deleted_model.id
                            {
                                item_val.set(None);
                                break;
                            }
                        }
                    });
                    None
                })
            }
        })),
        labels: ResourceTableLabels {
            save: "Save".to_string(),
            cancel: "Cancel".to_string(),
            edit: "Edit".to_string(),
            delete: "Delete".to_string(),
            confirm_delete: "Confirm".to_string(),
            confirm_question: "Are you sure?".to_string(),
            processing: "Processing...".to_string(),
        },
    };

    table.mount()
}
