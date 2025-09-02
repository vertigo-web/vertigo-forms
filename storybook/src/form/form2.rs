use std::rc::Rc;

use vertigo::{bind_rc, component, css, dom, Value};
use vertigo_forms::form::{DataSection, FormData, FormExport, FormParams, ModelForm};

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

        Self::default()
            .with(DataSection::with_string_field(
                "First Name",
                "first_name",
                &value.first_name,
            ))
            .with(DataSection::with_string_field(
                "Surname",
                "surname",
                &value.surname,
            ))
            .with(DataSection::new("Gender").add_list_field(
                "gender",
                Some(value.gender.to_string()),
                gender_map,
            ))
            .with(DataSection::new("Role").add_static_dict_field(
                "role",
                Some(value.role),
                role_map,
            ))
            .with(DataSection::new("Photo").add_image_field("photo", Some(&value.photo)))
            .add_bottom_controls()
    }
}

impl From<FormExport> for MySecondModel {
    fn from(form_export: FormExport) -> Self {
        Self {
            first_name: form_export.get_string("first_name"),
            surname: form_export.get_string("surname"),
            gender: form_export.list("gender").unwrap_or(Gender::Male),
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
                    submit_label: Rc::new("Apply".to_string()),
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
