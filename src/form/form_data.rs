use std::{collections::HashMap, rc::Rc};
use vertigo::{transaction, Computed, Context, DomElement, DropFileItem, Value};

#[derive(Clone)]
pub struct StringValue {
    pub value: Value<String>,
    pub original_value: Rc<String>,
}

#[derive(Clone)]
pub struct ListValue {
    pub value: Value<String>,
    pub original_value: String,
    pub dict: Computed<Vec<String>>,
}

#[derive(Clone)]
pub enum DataFieldValue {
    String(StringValue),
    List(ListValue),
    File(Value<DropFileItem>),
}

impl DataFieldValue {
    pub fn export(&self, ctx: &Context) -> FieldExport {
        match self {
            Self::String(val) => FieldExport::String(val.value.get(ctx)),
            Self::List(val) => FieldExport::List(val.value.get(ctx)),
            Self::File(val) => FieldExport::File(val.get(ctx)),
        }
    }
}

pub enum FieldExport {
    String(String),
    List(String),
    File(DropFileItem),
}

pub struct FormExport(HashMap<String, FieldExport>);

impl FormExport {
    pub fn get<'a>(&'a self, key: &str) -> Option<&'a FieldExport> {
        self.0.get(key)
    }

    pub fn get_string(&self, key: &str) -> String {
        self.get(key)
            .map(|export| {
                if let FieldExport::String(val) = export {
                    val.clone()
                } else {
                    Default::default()
                }
            })
            .unwrap_or_default()
    }

    pub fn list_or_default(&self, key: &str) -> String {
        self.get(key)
            .map(|export| {
                if let FieldExport::List(val) = export {
                    val.clone()
                } else {
                    Default::default()
                }
            })
            .unwrap_or_default()
    }

    pub fn file(&self, key: &str) -> Option<DropFileItem> {
        self.get(key)
            .and_then(|export| {
                if let FieldExport::File(val) = export {
                    Some(val.clone())
                } else {
                    None
                }
            })
    }
}

/// Presets for rendering fields in fieldset
#[derive(Clone, Copy, Default, PartialEq)]
pub enum FieldsetStyle {
    /// Just one after another (piled)
    #[default]
    Plain,
    /// Interspersed with "x" character
    Dimensions,
}

#[derive(Default)]
pub struct DataSection {
    pub label: String,
    pub fields: Vec<DataField>,
    pub error: Option<String>,
    pub render: Option<Rc<dyn Fn(Vec<DataField>) -> DomElement>>,
    pub fieldset_style: FieldsetStyle,
}

pub struct DataField {
    pub key: String,
    pub value: DataFieldValue,
}

impl DataSection {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    pub fn with_string_field(
        label: impl Into<String>,
        key: impl Into<String>,
        original_value: impl Into<String>,
    ) -> Self {
        let value = original_value.into();
        Self {
            label: label.into(),
            fields: vec![DataField {
                key: key.into(),
                value: DataFieldValue::String(StringValue {
                    value: Value::new(value.clone()),
                    original_value: Rc::new(value),
                }),
            }],
            ..Default::default()
        }
    }

    pub fn add_string_field(mut self, key: impl Into<String>, original_value: impl Into<String>) -> Self {
        let value = original_value.into();
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::String(StringValue {
                value: Value::new(value.clone()),
                original_value: Rc::new(value),
            }),
        });
        self
    }

    pub fn add_list_field(mut self, key: impl Into<String>, original_value: impl Into<String>, dict: Vec<String>) -> Self {
        let value = original_value.into();
        let dict = Computed::from(move |_ctx| dict.clone());
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::List(ListValue {
                value: Value::new(value.clone()),
                original_value: value,
                dict,
            }),
        });
        self
    }

    pub fn add_fiel_field(mut self, key: impl Into<String>, original_value: impl Into<String>) -> Self {
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::File(
                Value::new(
                    DropFileItem {
                        name: original_value.into(),
                        data: Rc::new(Vec::new()), // TODO
                    }
                )
            ),
        });
        self
    }

    pub fn set_fieldset_style(mut self, fieldset_style: FieldsetStyle) -> Self {
        self.fieldset_style = fieldset_style;
        self
    }
}

pub struct FormData {
    pub sections: Vec<DataSection>,
}

impl FormData {
    pub fn export(&self) -> FormExport {
        let mut hash_map = HashMap::new();
        transaction(|ctx| {
            for section in &self.sections {
                for field in &section.fields {
                    hash_map.insert(field.key.clone(), field.value.export(ctx));
                }
            }
        });
        FormExport(hash_map)
    }
}
