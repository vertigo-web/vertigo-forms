use std::{collections::HashMap, rc::Rc};
use vertigo::DropFileItem;

use crate::{image_as_uri, nonify};

pub enum FieldExport {
    Bool(bool),
    String(String),
    List(String),
    Dict(i64),
    Multi(Vec<i64>),
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
        self.get_string_opt(key).unwrap_or_default()
    }

    /// Get value from string input or None if empty.
    pub fn get_string_opt(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|export| {
            if let FieldExport::String(val) = export {
                nonify(val.clone())
            } else {
                None
            }
        })
    }

    // Get optional value from list field (string-based).
    pub fn list<T: From<String>>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|export| {
            if let FieldExport::List(val) = export {
                nonify(val.clone()).map(Into::into)
            } else {
                None
            }
        })
    }
    // Get value from list field (string-based) or default.
    pub fn list_or_default<T: Default + From<String>>(&self, key: &str) -> T {
        self.list(key).unwrap_or_default()
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

    // Get values from multi field (i64-based).
    pub fn multi<T: From<i64>>(&self, key: &str) -> Vec<T> {
        self.get(key)
            .map(|export| {
                if let FieldExport::Multi(val) = export {
                    val.iter().cloned().map(Into::into).collect()
                } else {
                    Default::default()
                }
            })
            .unwrap_or_default()
    }

    /// Get value from bool input (i. e. checkbox) or false.
    pub fn get_bool(&self, key: &str) -> bool {
        self.get_bool_opt(key).unwrap_or_default()
    }

    /// Get value from bool input (i. e. checkbox) or None.
    pub fn get_bool_opt(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|export| {
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
                FieldExport::Image((orig_link, dfi)) => dfi
                    .as_ref()
                    .map(image_as_uri)
                    .or_else(|| orig_link.as_ref().map(|ol| ol.to_string())),
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
                FieldExport::Image((_orig_link, dfi)) => dfi.clone(),
                _ => None,
            }
        } else {
            None
        }
    }

    // Get original image url
    pub fn image_orig_url_opt(&self, key: &str) -> Option<Rc<String>> {
        if let Some(export) = self.get(key) {
            match export {
                FieldExport::Image((orig_link, _dfi)) => orig_link.clone(),
                _ => None,
            }
        } else {
            None
        }
    }
}
