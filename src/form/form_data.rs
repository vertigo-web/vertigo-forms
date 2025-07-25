use std::{collections::HashMap, rc::Rc};
use vertigo::{transaction, Computed, Css, DomElement, Value};

use crate::form::field::BoolValue;

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
    pub top_controls: ControlsConfig,
    pub bottom_controls: ControlsConfig,
}

#[derive(Default)]
pub struct ControlsConfig {
    pub css: Option<Css>,
    pub submit: bool,
    pub delete: bool,
}

impl ControlsConfig {
    pub fn full() -> Self {
        Self {
            css: None,
            submit: true,
            delete: true,
        }
    }

    pub fn with_css(mut self, css: Css) -> Self {
        self.css = Some(css);
        self
    }
}

impl FormData {
    /// Add new data section
    pub fn with(mut self, section: DataSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn add_top_controls(mut self) -> Self {
        self.top_controls = ControlsConfig::full();
        self
    }

    pub fn add_top_controls_styled(mut self, css: Css) -> Self {
        self.top_controls = ControlsConfig::full().with_css(css);
        self
    }

    pub fn add_bottom_controls(mut self) -> Self {
        self.bottom_controls = ControlsConfig::full();
        self
    }

    pub fn add_bottom_controls_styled(mut self, css: Css) -> Self {
        self.bottom_controls = ControlsConfig::full().with_css(css);
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
    pub fieldset_css: Option<Css>,
    pub new_group: bool,
}

/// A single field in form section.
#[derive(Clone)]
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

    pub fn add_field(mut self, key: impl Into<String>, value: DataFieldValue) -> Self {
        self.fields.push(DataField {
            key: key.into(),
            value,
        });
        self
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
        original_value: Option<impl Into<String>>,
        options: Vec<String>,
    ) -> Self {
        let value = original_value.map(|s| s.into());
        let options = Computed::from(move |_ctx| options.clone());
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::List(ListValue {
                value: Value::new(value.clone().unwrap_or_default()),
                original_value: value.map(Rc::new),
                options,
            }),
        });
        self
    }

    /// Add another dict field to form section (dropdown with options, value stored as integer).
    pub fn add_dict_field(
        mut self,
        key: impl Into<String>,
        original_value: Option<i64>,
        options: Vec<(i64, String)>,
    ) -> Self {
        let options = Computed::from(move |_ctx| options.clone());
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::Dict(DictValue {
                value: Value::new(original_value.unwrap_or_default()),
                original_value: original_value.map(Rc::new),
                options,
            }),
        });
        self
    }

    /// Add another bool field to form section (checkbox input).
    pub fn add_bool_field(
        mut self,
        key: impl Into<String>,
        original_value: Option<impl Into<bool>>,
    ) -> Self {
        let value = original_value.map(|b| b.into());
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::Bool(BoolValue {
                value: Value::new(value.unwrap_or_default()),
                original_value: value.map(Rc::new),
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
        let value = original_value.map(|l| l.into());
        self.fields.push(DataField {
            key: key.into(),
            value: DataFieldValue::Image(ImageValue {
                value: Value::new(None),
                original_link: value.map(Rc::new),
                component_params: None,
            }),
        });
        self
    }

    /// Set [FieldsetStyle] for this section.
    pub fn set_fieldset_style(mut self, fieldset_style: FieldsetStyle) -> Self {
        self.fieldset_style = fieldset_style;
        self
    }

    /// Set [Css] for fields container for this section.
    pub fn set_fieldset_css(mut self, fieldset_css: Css) -> Self {
        self.fieldset_css = Some(fieldset_css);
        self
    }

    /// This section starts a new section group (Form adds a horizontal rule)
    pub fn starts_new_group(mut self) -> Self {
        self.new_group = true;
        self
    }
}
