use std::rc::Rc;
use vertigo::{DomNode, Value, css, dom};
use vertigo_forms::{
    Input,
    resource_table::{
        ResourceTable, ResourceTableLabels, base_header_css, base_row_css, main_col_css,
        normal_col_css,
    },
};

use super::fake_server::{FakeServer, Item};

#[derive(Clone, PartialEq, Default)]
struct MyModel {
    id: u32,
    name: String,
}

impl Item for MyModel {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}

#[derive(Clone)]
struct MyModelForm {
    name: Value<String>,
}

pub fn resource_table_text_field(server_rejecting: &Value<bool>) -> DomNode {
    let server = FakeServer::new(
        vec![
            MyModel {
                id: 1,
                name: "Item 1".to_string(),
            },
            MyModel {
                id: 2,
                name: "Item 2".to_string(),
            },
        ],
        server_rejecting,
    );

    let table = ResourceTable {
        list: server.list(),
        title: "My Resources".to_string(),
        add_label: "Add Item".to_string(),
        table_css: css! {""},
        render_header: || {
            dom! {
                <div css={base_header_css() + css! {"grid-template-columns: 50px 1fr 150px;"}}>
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
                <div css={base_row_css() + css! {"grid-template-columns: 50px 1fr 150px;"}}>
                    <div css={normal_col_css()}>{model.id}</div>
                    <div css={main_col_css()}>{model.name.clone()}</div>
                    <div>{create_buttons()}</div>
                </div>
            }
        },
        render_row_form: |form, buttons| {
            dom! {
                <div css={base_row_css() + css! {"grid-template-columns: 50px 1fr 150px;"}}>
                    <div>"-"</div>
                    <div>
                        <Input value={form.name.clone()} />
                    </div>
                    <div>{buttons}</div>
                </div>
            }
        },
        on_create: server.on_create(),
        on_update: server.on_update(),
        on_delete: Some(server.on_delete()),
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
