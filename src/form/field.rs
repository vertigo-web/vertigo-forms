use std::{collections::HashMap, rc::Rc};
use vertigo::{Computed, Context, DropFileItem, Value};

use crate::image_as_uri;

#[derive(Clone)]
pub struct StringValue {
    pub value: Value<String>,
    pub original_value: Rc<String>,
}

#[derive(Clone)]
pub struct ListValue {
    pub value: Value<String>,
    pub original_value: Rc<String>,
    pub options: Computed<Vec<String>>,
}

#[derive(Clone)]
pub struct DictValue {
    pub value: Value<i64>,
    pub original_value: Rc<i64>,
    pub options: Computed<Vec<(i64, String)>>,
}

#[derive(Clone)]
pub struct ImageValue {
    pub value: Value<Option<DropFileItem>>,
    pub original_link: Option<Rc<String>>,
}

/// Value of a field in form section.
#[derive(Clone)]
pub enum DataFieldValue {
    /// Regular string field.
    String(StringValue),
    /// String field with options.
    List(ListValue),
    /// Integer (foreign key) field with labels for each integer.
    Dict(DictValue),
    /// Image (bytes) field.
    Image(ImageValue),
}

impl DataFieldValue {
    pub fn export(&self, ctx: &Context) -> FieldExport {
        match self {
            Self::String(val) => FieldExport::String(val.value.get(ctx)),
            Self::List(val) => FieldExport::List(val.value.get(ctx)),
            Self::Dict(val) => FieldExport::Dict(val.value.get(ctx)),
            Self::Image(val) => FieldExport::Image((val.original_link.clone(), val.value.get(ctx))),
        }
    }
}

pub enum FieldExport {
    String(String),
    List(String),
    Dict(i64),
    Image((Option<Rc<String>>, Option<DropFileItem>)),
}

/// After form is submitted, it generates an export from every field. This can be used to construct a new model.
pub struct FormExport(HashMap<String, FieldExport>);

impl FormExport {
    pub fn new(map: HashMap<String, FieldExport>) -> Self {
        Self(map)
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a FieldExport> {
        self.0.get(key)
    }

    /// Get value from string input.
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

    // Get value from list field (string).
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

    // Get value from dict field (i64).
    pub fn dict_or_default(&self, key: &str) -> i64 {
        self.get(key)
            .map(|export| {
                if let FieldExport::Dict(val) = export {
                    *val
                } else {
                    Default::default()
                }
            })
            .unwrap_or_default()
    }

    // Get new image (base64) or original value from image field.
    pub fn image_url(&self, key: &str) -> String {
        if let Some(export) = self.get(key) {
            match export {
                FieldExport::Image((orig_link, dfi)) => {
                    dfi.as_ref().map(image_as_uri).unwrap_or_else(|| {
                        orig_link
                            .as_ref()
                            .map(|ol| ol.to_string())
                            .unwrap_or_default()
                    })
                }
                _ => String::default(),
            }
        } else {
            String::default()
        }
    }
}
