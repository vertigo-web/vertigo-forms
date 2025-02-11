use std::rc::Rc;

use vertigo::{bind_rc, css, dom, transaction, DomNode, Value};
use vertigo_forms::{FieldsetStyle, Form, FormData, FormParams, FormSection};

#[derive(Clone, PartialEq)]
pub struct MyModel {
    pub slug: String,
    pub name: String,
    pub dimension_x: String,
    pub dimension_y: String,
}

impl From<&MyModel> for FormData {
    fn from(value: &MyModel) -> Self {
        Self {
            sections: vec![
                FormSection::new("Slug", "slug", &value.slug),
                FormSection::new("Name", "name", &value.name),
                FormSection::new("Dimensions", "dimension_x", &value.dimension_x)
                    .add_field("dimension_y", &value.dimension_y)
                    .set_fieldset_style(FieldsetStyle::Dimensions),
            ],
        }
    }
}

impl From<Rc<FormData>> for MyModel {
    fn from(form_data: Rc<FormData>) -> Self {
        transaction(|ctx| Self {
            slug: form_data.get("slug").get(ctx),
            name: form_data.get("name").get(ctx),
            dimension_x: form_data.get("dimension_x").get(ctx),
            dimension_y: form_data.get("dimension_y").get(ctx),
        })
    }
}

pub fn form() -> DomNode {
    let my_model: Value<MyModel> = Value::new(MyModel {
        slug: "model-one".to_string(),
        name: "Model One".to_string(),
        dimension_x: "120".to_string(),
        dimension_y: "80".to_string(),
    });

    let my_model_clone = my_model.clone();
    let form = my_model.render_value(move |model| {
        let on_submit = bind_rc!(my_model_clone, |new_model: MyModel| {
            my_model_clone.set(new_model);
        });

        dom! {
            <Form
                model={&&model}
                {on_submit}
                params={FormParams {
                    add_css: css! {"width: 400px;"},
                    submit_label: "Apply".to_string(),
                    ..Default::default()
                }}
            />
        }
    });

    dom! {
        <div>
            <h4>"Form:"</h4>
            {form}
            <h4>"Model:"</h4>
            <p>{my_model.map(|m| m.slug)} " / " {my_model.map(|m| m.name)}</p>
            <p>{my_model.map(|m| m.dimension_x)} "x" {my_model.map(|m| m.dimension_y)}</p>
        </div>
    }
}
