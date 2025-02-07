use std::{collections::HashMap, rc::Rc};
use vertigo::{transaction, Computed, DomElement, Value};

use super::{
    field::{DictValue, ImageValue, ListValue, StringValue},
    DataFieldValue, FormExport,
};

/// Used to define structure of a [Form](super::Form).
///
/// Example:
///
/// ```rust
/// use vertigo_forms::form::{DataSection, FieldsetStyle, FormData};
///
/// #[derive(Clone, PartialEq)]
/// pub struct MyModel {
///     pub slug: String,
///     pub name: String,
///     pub dimension_x: String,
///     pub dimension_y: String,
/// }
///
/// impl From<&MyModel> for FormData {
///     fn from(value: &MyModel) -> Self {
///         Self::default()
///             .with(DataSection::with_string_field("Slug", "slug", &value.slug))
///             .with(DataSection::with_string_field("Name", "name", &value.name))
///             .with(
///                 DataSection::with_string_field("Dimensions", "dimension_x", &value.dimension_x)
///                     .add_string_field("dimension_y", &value.dimension_y)
///                     .set_fieldset_style(FieldsetStyle::Dimensions),
///             )
///     }
/// }
/// ```
///
/// See story book for more examples.
#[derive(Default)]
pub struct FormData {
    pub sections: Vec<DataSection>,
}

impl FormData {
    /// Add new data section
    pub fn with(mut self, section: DataSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn export(&self) -> FormExport {
        let mut hash_map = HashMap::new();
        transaction(|ctx| {
            for section in &self.sections {
                for field in &section.fields {
                    hash_map.insert(field.key.clone(), field.value.export(ctx));
                }
            }
        });
        FormExport::new(hash_map)
    }
}

/// Presets for rendering fields in a field set.
#[derive(Clone, Copy, Default, PartialEq)]
pub enum FieldsetStyle {
    /// Just one after another (piled)
    #[default]
    Plain,
    /// Interspersed with "x" character
    Dimensions,
}

/// A section of form with label and a field (or field set).
#[derive(Default)]
pub struct DataSection {
    pub label: String,
    pub fields: Vec<DataField>,
    pub error: Option<String>,
    pub render: Option<Rc<dyn Fn(Vec<DataField>) -> DomElement>>,
    pub fieldset_style: FieldsetStyle,
}

/// A single field in form section.
pub struct DataField {
    pub key: String,
    pub value: DataFieldValue,
}

impl DataSection {
    /// Create a new form section without fields.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    /// Create a new form section with single string field.
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

    /// Add another string field to form section (text input).
    pub fn add_string_field(
        mut self,
        key: impl Into<String>,
        original_value: impl Into<String>,
    ) -> Self {
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

    /// Add another list field to form section (dropdown with options).
    pub fn add_list_field(
        mut self,
        key: impl Into<String>,
        original_value: impl Into<String>,
        options: Vec<String>,
    ) -> Self {
        let value = original_value.into();
        let options = Computed::from(move |_ctx| options.clone());
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::List(ListValue {
                value: Value::new(value.clone()),
                original_value: Rc::new(value),
                options,
            }),
        });
        self
    }

    /// Add another dict field to form section (dropdown with options, value stored as integer).
    pub fn add_dict_field(
        mut self,
        key: impl Into<String>,
        original_value: i64,
        options: Vec<(i64, String)>,
    ) -> Self {
        let options = Computed::from(move |_ctx| options.clone());
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::Dict(DictValue {
                value: Value::new(original_value),
                original_value: Rc::new(original_value),
                options,
            }),
        });
        self
    }

    /// Add another image field to form section.
    pub fn add_image_field(
        mut self,
        key: impl Into<String>,
        original_value: Option<impl Into<String>>,
    ) -> Self {
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::Image(ImageValue {
                value: Value::new(None),
                original_link: original_value.map(|link| Rc::new(link.into())),
            }),
        });
        self
    }

    /// Set [FieldsetStyle] for this section.
    pub fn set_fieldset_style(mut self, fieldset_style: FieldsetStyle) -> Self {
        self.fieldset_style = fieldset_style;
        self
    }
}
