use vertigo::{bind_rc, component, css, dom, DomNode, Value};
use vertigo_forms::form::{DataSection, FieldsetStyle, FormData, FormExport, FormParams, ModelForm};

pub fn form() -> DomNode {
    dom! {
        <Form1 />
        <hr />
        <Form2 />
    }
}

// Form example 1

#[derive(Clone, PartialEq)]
pub struct MyModel {
    pub slug: String,
    pub name: String,
    pub dimension_x: String,
    pub dimension_y: String,
}

impl From<&MyModel> for FormData {
    fn from(value: &MyModel) -> Self {
        Self::default()
            .with(DataSection::with_string_field("Slug", "slug", &value.slug))
            .with(DataSection::with_string_field("Name", "name", &value.name))
            .with(
                DataSection::with_string_field("Dimensions", "dimension_x", &value.dimension_x)
                    .add_string_field("dimension_y", &value.dimension_y)
                    .set_fieldset_style(FieldsetStyle::Dimensions),
            )
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

    let my_model_clone = my_model.clone();
    let form = my_model.render_value(move |model| {
        let on_submit = bind_rc!(my_model_clone, |new_model: MyModel| {
            my_model_clone.set(new_model);
        });

        dom! {
            <ModelForm
                model={&&model}
                {on_submit}
                params={FormParams {
                    add_css: css! {"width: 400px;"},
                    ..Default::default()
                }}
            />
        }
    });

    dom! {
        <div>
            <h4>"Form 1:"</h4>
            {form}
            <h4>"Model 1:"</h4>
            <p>{my_model.map(|m| m.slug)} " / " {my_model.map(|m| m.name)}</p>
            <p>{my_model.map(|m| m.dimension_x)} "x" {my_model.map(|m| m.dimension_y)}</p>
        </div>
    }
}

// Form example 2

#[derive(Clone, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Male => write!(f, "Male"),
            Self::Female => write!(f, "Female"),
        }
    }
}

impl From<String> for Gender {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Female" => Self::Female,
            _ => Self::Male,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct MySecondModel {
    pub first_name: String,
    pub surname: String,
    pub gender: Gender,
    pub role: i64,
    pub photo: String,
}

impl From<&MySecondModel> for FormData {
    fn from(value: &MySecondModel) -> Self {
        let gender_map = vec!["Male".to_string(), "Female".to_string()];

        let role_map = [
            (1i64, "Admin".to_string()),
            (2, "Editor".to_string()),
            (3, "Reporter".to_string()),
            (4, "Viewer".to_string()),
        ]
        .into();

        Self {
            sections: vec![
                DataSection::with_string_field("First Name", "first_name", &value.first_name),
                DataSection::with_string_field("Surname", "surname", &value.surname),
                DataSection::new("Gender").add_list_field(
                    "gender",
                    value.gender.to_string(),
                    gender_map,
                ),
                DataSection::new("Role").add_dict_field("role", value.role, role_map),
                DataSection::new("Photo").add_image_field("photo", Some(&value.photo)),
            ],
        }
    }
}

impl From<FormExport> for MySecondModel {
    fn from(form_export: FormExport) -> Self {
        Self {
            first_name: form_export.get_string("first_name"),
            surname: form_export.get_string("surname"),
            gender: Gender::from(form_export.list_or_default("gender")),
            role: form_export.dict_or_default("role"),
            photo: form_export.image_url("photo"),
        }
    }
}

#[component]
pub fn Form2() {
    let my_second_model: Value<MySecondModel> = Value::new(MySecondModel {
        first_name: "Johann".to_string(),
        surname: "Gambolputty".to_string(),
        gender: Gender::Male,
        role: 1,
        photo: "https://picsum.photos/200".to_string(),
    });

    let my_model_clone = my_second_model.clone();
    let form = my_second_model.render_value(move |model| {
        let on_submit = bind_rc!(my_model_clone, |new_model: MySecondModel| {
            my_model_clone.set(new_model);
        });

        dom! {
            <ModelForm
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
            <h4>"Form 2:"</h4>
            {form}
            <h4>"Model 2:"</h4>
            <p>
                {my_second_model.map(|m| m.first_name)}
                " / "
                {my_second_model.map(|m| m.surname)}
                " (" {my_second_model.map(|m| m.gender.to_string())} ")"
            </p>
            <p>"Role: " {my_second_model.map(|m| m.role)}</p>
            <p>
                "Photo:" <br />
                <img src={my_second_model.map(|m| m.photo) } />
            </p>
        </div>
    }
}
