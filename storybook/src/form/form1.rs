use vertigo::{bind_rc, component, css, dom, Value};
use vertigo_forms::form::{
    DataSection, FieldsetStyle, FormData, FormExport, FormParams, ModelForm,
};

// Form example 1

#[derive(Clone, PartialEq)]
pub struct MyModel {
    pub slug: String,
    pub name: String,
    pub dimension_x: String,
    pub dimension_y: String,
}

impl From<MyModel> for FormData {
    fn from(value: MyModel) -> Self {
        Self::default()
            .with(DataSection::with_string_field("Slug", "slug", &value.slug))
            .with(DataSection::with_string_field("Name", "name", &value.name))
            .with(
                DataSection::with_string_field("Dimensions", "dimension_x", &value.dimension_x)
                    .add_string_field("dimension_y", &value.dimension_y)
                    .set_fieldset_style(FieldsetStyle::Dimensions),
            )
            .add_bottom_controls()
    }
}

impl From<FormExport> for MyModel {
    fn from(form_export: FormExport) -> Self {
        Self {
            slug: form_export.get_string("slug"),
            name: form_export.get_string("name"),
            dimension_x: form_export.get_string("dimension_x"),
            dimension_y: form_export.get_string("dimension_y"),
        }
    }
}

#[component]
pub fn Form1() {
    let my_model: Value<MyModel> = Value::new(MyModel {
        slug: "model-one".to_string(),
        name: "Model One".to_string(),
        dimension_x: "120".to_string(),
        dimension_y: "80".to_string(),
    });

    let on_submit = bind_rc!(my_model, |new_model: MyModel| {
        my_model.set(new_model);
    });

    dom! {
        <div>
            <h4>"Form 1:"</h4>
            <ModelForm
                model={my_model.clone()}
                {on_submit}
                params={FormParams {
                    add_css: css! {"width: 400px;"},
                    ..Default::default()
                }}
            />
            <h4>"Model 1:"</h4>
            <p>{my_model.map(|m| m.slug)} " / " {my_model.map(|m| m.name)}</p>
            <p>{my_model.map(|m| m.dimension_x)} "x" {my_model.map(|m| m.dimension_y)}</p>
        </div>
    }
}
