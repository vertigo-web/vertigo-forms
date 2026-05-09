use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD as BASE_64};
use vertigo::DropFileItem;

use crate::name_to_mime;

#[derive(Clone, Debug, PartialEq, vertigo::AutoJsJson)]
pub struct ImageUpload {
    pub name: String,
    pub mime: Option<String>,
    pub data: String, // base64
}

impl ImageUpload {
    pub fn to_vec(&self) -> Result<Vec<u8>, base64::DecodeError> {
        BASE_64.decode(&self.data)
    }
}

impl From<DropFileItem> for ImageUpload {
    fn from(item: DropFileItem) -> Self {
        let mime_str = name_to_mime(&item.name);
        let mime = if mime_str == "application/octet-stream" {
            None
        } else {
            Some(mime_str.to_string())
        };
        Self {
            name: item.name.clone(),
            mime,
            data: BASE_64.encode(&*item.data),
        }
    }
}
