use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD as BASE_64};
use std::rc::Rc;
use vertigo::{DomNode, DropFileItem, DropFileEvent, Value, css, dom, computed_tuple, Computed, bind, Css};

/// Box that allows to accept image files on it, connected to `Value<Option<DropFileItem>>`.
pub struct DropImageFile {
    pub original_link: Computed<Option<String>>,
    pub item: Value<Option<DropFileItem>>,
    pub params: DropImageFileParams,
}

#[derive(Clone)]
pub struct DropImageFileParams {
    pub callback: Option<Rc<dyn Fn(DropFileItem)>>,
    pub revert_label: String,
    pub cancel_label: String,
    pub no_image_text: String,
    pub dropzone_css: Css,
    pub dropzone_add_css: Css,
}

impl Default for DropImageFileParams {
    fn default() -> Self {
        Self {
            callback: None,
            revert_label: "Revert".to_string(),
            cancel_label: "Cancel".to_string(),
            no_image_text: "No image".to_string(),
            dropzone_css: css!("
                width: 400px;
                height: 400px;

                display: flex;
                align-items: center;
                justify-content: center;

                padding: 10px;
            "),
            dropzone_add_css: css!(""),
        }
    }
}

impl DropImageFile {
    pub fn mount(&self) -> DomNode {
        let base64_data = self.item.to_computed().map(|item| {
            match item {
                Some(item) => image_as_uri(&item),
                None => "".to_string()
            }
        });

        let view_deps = computed_tuple!(
            a => self.original_link,
            b => self.item,
            c => base64_data
        );
        let item_clone = self.item.clone();
        let params = self.params.clone();
        let image_view = view_deps.render_value(
            move |(original, item, base64_date)| {
                match item {
                    Some(item) => {
                        let message = format_line(&item);
                        let image_css = css!("
                            display: flex;
                            flex-flow: column;
                        ");
                        let restore = bind!(item_clone, || item_clone.set(None));
                        let restore_text = if original.is_some() {
                            &params.revert_label
                        } else {
                            &params.cancel_label
                        };
                        dom! {
                            <div css={image_css}>
                                <button on_click={restore}>{restore_text}</button>
                                <img src={base64_date} />
                                { message }
                            </div>
                        }
                    }
                    None => match original {
                        Some(original) => { dom! { <div><img src={original} /></div> } },
                        None => dom! { <div>{&params.no_image_text}</div> }
                    },
                }
            }
        );

        let item = self.item.clone();
        let callback = self.params.callback.clone();
        let on_dropfile = move |event: DropFileEvent| {
            for file in event.items.into_iter() {
                if let Some(callback) = callback.as_deref() {
                    callback(file);
                } else {
                    item.change(|current| {
                        *current = Some(file);
                    });
                }
            }
        };

        let dropzone_css = self.params.dropzone_css.clone().extend(self.params.dropzone_add_css.clone());

        dom! {
            <div css={dropzone_css} on_dropfile={on_dropfile}>
                { image_view }
            </div>
        }
    }
}

fn format_line(item: &DropFileItem) -> String {
    let file_name = &item.name;
    let size = item.data.len();
    format!("{file_name} ({size})")
}

pub fn name_to_mime(name: &str) -> &'static str {
    use std::{ffi::OsStr, path::Path};

    let extension = Path::new(name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default();

    match extension {
        "jpg" | "jpeg" | "jpe" => "image/jpeg",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "ico" => "image/ico",
        _ => "application/octet-stream",
    }
}

pub fn image_as_uri(item: &DropFileItem) -> String {
    let mime = name_to_mime(&item.name);
    let data = BASE_64.encode(&*item.data);
    format!("data:{mime};base64,{data}")
}
