use std::{collections::HashMap, rc::Rc};
use vertigo::{Computed, Context, DomNode, DropFileItem, Value};

use crate::DropImageFileParams;

use super::form_export::FieldExport;

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
    /// Array of integers (foreign key) field with labels for each integer.
    Multi(MultiValue),
    /// Checkbox
    Bool(BoolValue),
    /// Image (bytes) field.
    Image(ImageValue),
    /// Custom field
    Custom(CustomValue),
    /// Custom component without value
    StaticCustom(Rc<dyn Fn() -> DomNode>),
}

impl DataFieldValue {
    pub fn export(&self, ctx: &Context) -> FieldExport {
        match self {
            Self::String(val) => FieldExport::String(val.value.get(ctx)),
            Self::TextArea(val) => FieldExport::String(val.value.get(ctx)),
            Self::List(val) => FieldExport::List(val.value.get(ctx)),
            Self::Dict(val) => FieldExport::Dict(val.value.get(ctx)),
            Self::Multi(val) => {
                FieldExport::Multi(val.value.get(ctx).iter().map(|v| v.get(ctx)).collect())
            }
            Self::Bool(val) => FieldExport::Bool(val.value.get(ctx)),
            Self::Image(val) => FieldExport::Image((val.original_link.clone(), val.value.get(ctx))),
            Self::Custom(val) => FieldExport::String(val.value.get(ctx)),
            Self::StaticCustom(_) => FieldExport::String("".to_string()),
        }
    }
}

/// Value of a [DataFieldValue::String] field.
///
/// Built by hand, a field can share its `Value` with something else, f. ex. a preview that
/// follows what is typed. [DataSection::with_string_field](super::DataSection::with_string_field)
/// creates a `Value` of its own instead.
///
/// ```rust
/// use std::rc::Rc;
/// use vertigo::{Value, transaction};
/// use vertigo_forms::form::{DataFieldValue, DataSection, StringValue};
///
/// let title = Value::new("Draft".to_string());
/// let section = DataSection::new("Title").add_field(
///     "title",
///     DataFieldValue::String(StringValue {
///         value: title.clone(),
///         original_value: Rc::new("Draft".to_string()),
///     }),
/// );
///
/// // Typing into the field changes `title` as well
/// let DataFieldValue::String(field) = &section.fields[0].value else { unreachable!() };
/// field.value.set("Final".to_string());
/// assert_eq!(transaction(|ctx| title.get(ctx)), "Final");
/// ```
#[derive(Clone)]
pub struct StringValue {
    pub value: Value<String>,
    pub original_value: Rc<String>,
}

/// Value of a [DataFieldValue::TextArea] field.
#[derive(Clone)]
pub struct TextAreaValue {
    pub value: Value<String>,
    pub original_value: Option<Rc<String>>,
    pub rows: Option<i32>,
    pub cols: Option<i32>,
}

/// Value of a [DataFieldValue::List] field.
#[derive(Clone)]
pub struct ListValue {
    pub value: Value<String>,
    pub original_value: Option<Rc<String>>,
    pub options: Computed<Vec<String>>,
}

/// Value of a [DataFieldValue::Dict] field.
#[derive(Clone)]
pub struct DictValue {
    pub value: Value<i64>,
    pub original_value: Option<Rc<i64>>,
    pub options: Computed<Vec<(i64, String)>>,
}

/// Value of a [DataFieldValue::Multi] field.
#[derive(Clone)]
pub struct MultiValue {
    pub value: Value<Vec<Value<i64>>>,
    pub original_value: Rc<Vec<i64>>,
    pub options: Computed<HashMap<i64, String>>,
    pub add_label: Rc<String>,
}

/// Value of a [DataFieldValue::Bool] field.
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
