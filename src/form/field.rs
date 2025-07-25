use std::{collections::HashMap, rc::Rc};
use vertigo::{Computed, Context, DomNode, DropFileItem, Value};

use crate::{image_as_uri, nonify, DropImageFileParams};

#[derive(Clone)]
pub struct StringValue {
    pub value: Value<String>,
    pub original_value: Rc<String>,
}

#[derive(Clone)]
pub struct TextAreaValue {
    pub value: Value<String>,
    pub original_value: Option<Rc<String>>,
    pub rows: Option<i32>,
    pub cols: Option<i32>,
}

#[derive(Clone)]
pub struct ListValue {
    pub value: Value<String>,
    pub original_value: Option<Rc<String>>,
    pub options: Computed<Vec<String>>,
}

#[derive(Clone)]
pub struct DictValue {
    pub value: Value<i64>,
    pub original_value: Option<Rc<i64>>,
    pub options: Computed<Vec<(i64, String)>>,
}

#[derive(Clone)]
pub struct BoolValue {
    pub value: Value<bool>,
    pub original_value: Option<Rc<bool>>,
}

#[derive(Clone)]
pub struct ImageValue {
    pub value: Value<Option<DropFileItem>>,
    pub original_link: Option<Rc<String>>,
    pub component_params: Option<DropImageFileParams>,
}

#[derive(Clone)]
pub struct CustomValue {
    pub value: Value<String>,
    pub original_value: Option<Rc<String>>,
    pub render: Rc<dyn Fn() -> DomNode>,
}

/// Value of a field in form section.
#[derive(Clone)]
pub enum DataFieldValue {
    /// Regular string field.
    String(StringValue),
    /// Textarea string field.
    TextArea(TextAreaValue),
    /// String field with options.
    List(ListValue),
    /// Integer (foreign key) field with labels for each integer.
    Dict(DictValue),
    /// Checkbox
    Bool(BoolValue),
    /// Image (bytes) field.
    Image(ImageValue),
    /// Custom field
    Custom(CustomValue),
    /// Custom component without value
    StaticCustom(Rc<dyn Fn() -> DomNode>)
}

impl DataFieldValue {
    pub fn export(&self, ctx: &Context) -> FieldExport {
        match self {
            Self::String(val) => FieldExport::String(val.value.get(ctx)),
            Self::TextArea(val) => FieldExport::String(val.value.get(ctx)),
            Self::List(val) => FieldExport::List(val.value.get(ctx)),
            Self::Dict(val) => FieldExport::Dict(val.value.get(ctx)),
            Self::Bool(val) => FieldExport::Bool(val.value.get(ctx)),
            Self::Image(val) => FieldExport::Image((val.original_link.clone(), val.value.get(ctx))),
            Self::Custom(val) => FieldExport::String(val.value.get(ctx)),
            Self::StaticCustom(_) => FieldExport::String("".to_string())
        }
    }
}

pub enum FieldExport {
    Bool(bool),
    String(String),
    List(String),
    Dict(i64),
    Image((Option<Rc<String>>, Option<DropFileItem>)),
}

/// After form is submitted, it generates an export from every field. This can be used to construct a new model.
#[derive(Clone)]
pub struct FormExport(Rc<HashMap<String, FieldExport>>);

impl FormExport {
    pub fn new(map: HashMap<String, FieldExport>) -> Self {
        Self(Rc::new(map))
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a FieldExport> {
        self.0.get(key)
    }

    /// Get value from string input.
    pub fn get_string(&self, key: &str) -> String {
        self.get_string_opt(key)
            .unwrap_or_default()
    }

    /// Get value from string input or None if empty.
    pub fn get_string_opt(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(|export| {
                if let FieldExport::String(val) = export {
                    nonify(val.clone())
                } else {
                    None
                }
            })
    }

    // Get value from list field (string-based).
    pub fn list_or_default<T: From<String>>(&self, key: &str) -> T {
        self.get(key)
            .map(|export| {
                if let FieldExport::List(val) = export {
                    val.clone()
                } else {
                    Default::default()
                }
            })
            .unwrap_or_default()
            .into()
    }

    // Get value from dict field (i64-based).
    pub fn dict_or_default<T: From<i64>>(&self, key: &str) -> T {
        self.get(key)
            .map(|export| {
                if let FieldExport::Dict(val) = export {
                    *val
                } else {
                    Default::default()
                }
            })
            .unwrap_or_default()
            .into()
    }

    /// Get value from bool input (i. e. checkbox) or false.
    pub fn get_bool(&self, key: &str) -> bool {
        self.get_bool_opt(key).unwrap_or_default()
    }

    /// Get value from bool input (i. e. checkbox) or None.
    pub fn get_bool_opt(&self, key: &str) -> Option<bool> {
        self.get(key)
            .and_then(|export| {
                if let FieldExport::Bool(val) = export {
                    Some(*val)
                } else {
                    None
                }
            })
    }

    // Get new image (base64) or original value from image field.
    pub fn image_url(&self, key: &str) -> String {
        self.image_url_opt(key).unwrap_or_default()
    }

    // Get new image (base64) or original value from image field or none.
    pub fn image_url_opt(&self, key: &str) -> Option<String> {
        if let Some(export) = self.get(key) {
            match export {
                FieldExport::Image((orig_link, dfi)) => {
                    dfi.as_ref().map(image_as_uri).or_else(|| {
                        orig_link
                            .as_ref()
                            .map(|ol| ol.to_string())
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }

    // Get new image (base64)
    pub fn image_item_opt(&self, key: &str) -> Option<DropFileItem> {
        if let Some(export) = self.get(key) {
            match export {
                FieldExport::Image((_orig_link, dfi)) => {
                    dfi.clone()
                }
                _ => None,
            }
        } else {
            None
        }
    }

    // Get original image url
    pub fn image_orig_url_opt(&self, key: &str) -> Option<String> {
        if let Some(export) = self.get(key) {
            match export {
                FieldExport::Image((orig_link, _dfi)) => {
                    orig_link.as_deref().map(|s| s.to_string())
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
