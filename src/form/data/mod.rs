use std::rc::Rc;
use vertigo::Value;

use crate::ValidationErrors;

mod data_field;
pub use data_field::{DataFieldValue, ImageValue, TextAreaValue};

mod form_export;
pub use form_export::{FieldExport, FormExport};

mod form_data;
pub use form_data::{ControlsConfig, DataField, DataSection, FieldsetStyle, FormData};

pub type ValidateFunc<T> = Rc<dyn Fn(&T, Value<ValidationErrors>) -> bool>;

#[derive(Default, Clone, PartialEq)]
pub enum Operation {
    #[default]
    None,
    Saving,
    Success,
    Error(Rc<String>),
}

impl Operation {
    pub fn err(message: impl Into<String>) -> Self {
        Self::Error(Rc::new(message.into()))
    }
}
