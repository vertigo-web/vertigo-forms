use std::rc::Rc;
use vertigo::{DomNode, Value, css, dom};
use vertigo_forms::{
    form::{DataFieldValue, DataSection},
    resource_table::{
        ResourceTable, ResourceTableLabels, base_header_css, base_row_css, main_col_css,
        normal_col_css, row_from_data_section,
    },
};

use super::fake_server::{FakeServer, Item};

#[derive(Clone, PartialEq, Default)]
struct ComplexModel {
    id: u32,
    name: String,
    is_active: bool,
    role: Option<i64>,
}

impl Item for ComplexModel {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}

pub fn resource_table_data_section(server_rejecting: &Value<bool>) -> DomNode {
    let server = FakeServer::new(
        vec![
            ComplexModel {
                id: 1,
                name: "Item 1".to_string(),
                is_active: true,
                role: Some(1),
            },
            ComplexModel {
                id: 2,
                name: "Item 2".to_string(),
                is_active: false,
                role: Some(2),
            },
        ],
        server_rejecting,
    );

    let table = ResourceTable {
        list: server.list(),
        title: "My Resources (DataSection)".to_string(),
        add_label: "Add Item".to_string(),
        table_css: css! {"margin-top: 40px;"},
        render_header: || {
            dom! {
                <div css={base_header_css() + css! {"grid-template-columns: 50px 1fr 100px 150px 150px;"}}>
                    <div>"ID"</div>
                    <div>"Name"</div>
                    <div>"Active"</div>
                    <div>"Role"</div>
                    <div>"Actions"</div>
                </div>
            }
        },
        render_filters: None,
        create_new_model: Rc::new(|| ComplexModel {
            id: 0,
            name: String::new(),
            is_active: false,
            role: None,
        }),
        create_form_model: |model| {
            Rc::new(
                DataSection::new(model.id.to_string())
                    .add_string_field("name", &model.name)
                    .add_bool_field("is_active", Some(model.is_active))
                    .add_static_dict_field(
                        "role",
                        model.role,
                        vec![(1, "Admin".to_string()), (2, "User".to_string())],
                    ),
            )
        },
        update_model: |model, form: &Rc<DataSection>, ctx| {
            let mut name = String::new();
            let mut is_active = false;
            let mut role = None;
            for field in &form.fields {
                match field.key.as_str() {
                    "name" => {
                        if let DataFieldValue::String(ref s) = field.value {
                            name = s.value.get(ctx);
                        }
                    }
                    "is_active" => {
                        if let DataFieldValue::Bool(ref b) = field.value {
                            is_active = b.value.get(ctx);
                        }
                    }
                    "role" => {
                        if let DataFieldValue::Dict(ref d) = field.value {
                            let val = d.value.get(ctx);
                            if val != 0 {
                                role = Some(val);
                            }
                        }
                    }
                    _ => {}
                }
            }
            if name.is_empty() {
                return Err(vec!["Name cannot be empty".to_string()]);
            }
            let mut new_model = model.clone();
            new_model.name = name;
            new_model.is_active = is_active;
            new_model.role = role;
            Ok(new_model)
        },
        render_row_view: |model, create_buttons, _alert| {
            let role_str = match model.role {
                Some(1) => "Admin",
                Some(2) => "User",
                _ => "Unknown",
            };
            let active_str = if model.is_active { "Yes" } else { "No" };
            dom! {
                <div css={base_row_css() + css! {"grid-template-columns: 50px 1fr 100px 150px 150px;"}}>
                    <div css={normal_col_css()}>{model.id}</div>
                    <div css={main_col_css()}>{model.name.clone()}</div>
                    <div css={normal_col_css()}>{active_str}</div>
                    <div css={normal_col_css()}>{role_str}</div>
                    <div>{create_buttons()}</div>
                </div>
            }
        },
        render_row_form: |form: &Rc<DataSection>, buttons| {
            row_from_data_section(form, buttons, "50px 1fr 100px 150px 150px")
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
