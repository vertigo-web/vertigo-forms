use std::rc::Rc;
use vertigo::{DomElement, Value};

// pub enum FieldKind {
// }

pub struct FormSection {
    pub label: String,
    pub fields: Vec<Field>,
    pub error: Option<String>,
    pub render: Option<Rc<dyn Fn(Vec<Field>) -> DomElement>>,
}

pub struct Field {
    pub key: String,
    pub value: Value<String>,
    pub original_value: String,
}

impl FormSection {
    pub fn new(
        label: impl Into<String>,
        key: impl Into<String>,
        original_value: impl Into<String>,
    ) -> Self {
        let value = original_value.into();
        Self {
            label: label.into(),
            fields: vec![Field {
                key: key.into(),
                value: Value::new(value.clone()),
                original_value: value,
            }],
            error: None,
            render: None,
        }
    }

    pub fn add_field(mut self, key: impl Into<String>, original_value: impl Into<String>) -> Self {
        let value = original_value.into();
        self.fields.push(Field {
            key: key.into(),
            value: Value::new(value.clone()),
            original_value: value,
        });
        self
    }
}

pub struct FormData {
    pub sections: Vec<FormSection>,
}

impl FormData {
    pub fn get(&self, key: &str) -> Value<String> {
        for section in &self.sections {
            for field in &section.fields {
                if field.key == key {
                    return field.value.clone();
                }
            }
        }
        Value::default()
    }
}
