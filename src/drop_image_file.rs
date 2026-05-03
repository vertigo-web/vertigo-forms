use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD as BASE_64};
use std::rc::Rc;
use vertigo::{
    AttrGroup, Computed, Css, DropFileEvent, DropFileItem, Value, bind, component, computed_tuple,
    css, dom,
};

/// Box that allows to accept image files on it, connected to `Value<Option<DropFileItem>>`.
#[component]
pub fn DropImageFile(
    original_link: Computed<Option<Rc<String>>>,
    item: Value<Option<DropFileItem>>,
    params: DropImageFileParams,
    /// Any additional attributes for the dropzone
    zone: AttrGroup,
) {
    let base64_data = item.to_computed().map(|item| match item {
        Some(item) => image_as_uri(&item),
        None => "".to_string(),
    });

    let view_deps = computed_tuple!(
        a => original_link,
        b => item,
        c => base64_data
    );
    let item_clone = item.clone();
    let params = params.clone();
    let callback = params.callback.clone();

    // Shared handler factory for the file-input-select path (cloned per render branch)
    let item_for_select = item.clone();
    let callback_for_select = params.callback.clone();
    let make_on_change_file = Rc::new(move || {
        let item = item_for_select.clone();
        let callback = callback_for_select.clone();
        move |event: DropFileEvent| {
            for file in event.items {
                if let Some(cb) = callback.as_deref() {
                    cb(Some(file));
                } else {
                    item.set(Some(file));
                }
            }
        }
    });

    let image_view = view_deps.render_value(move |(original, item, base64_date)| {
        let hidden_input_css = css! {"display: none;"};
        let btn_css = css! {"
            text-align: center;
            display: inline-block;
            cursor: pointer;
            padding: 4px 12px;
            border: 1px solid currentColor;
            border-radius: 4px;
            font-size: 0.85em;
            margin-top: 6px;
            margin-bottom: 6px;
        "};
        let select_label = params.select_label.clone();
        let accept = params.accept.clone();
        let flex_column = css! {"display: flex; flex-direction: column; align-items: center;"};

        match item {
            Some(item) => {
                let message = format_line(&item);
                let restore = bind!(item_clone, callback, |_| {
                    if let Some(callback) = &callback {
                        callback(None);
                    } else {
                        item_clone.set(None);
                    }
                });
                let restore_text = if original.is_some() {
                    &params.revert_label
                } else {
                    &params.cancel_label
                };
                let select_button = if select_label.is_empty() {
                    None
                } else {
                    Some(dom! {
                        <label css={&btn_css}>
                            <input
                                css={hidden_input_css}
                                type="file"
                                accept={accept}
                                on_change_file={make_on_change_file()}
                            />
                            {select_label}
                        </label>
                    })
                };
                dom! {
                    <div css={flex_column}>
                        <button css={btn_css}on_click={restore}>{restore_text}</button>
                        {..select_button}
                        <img css={&params.img_css} src={base64_date} />
                        { message }
                    </div>
                }
            }
            None => match original {
                Some(original) => {
                    if select_label.is_empty() {
                        dom! { <div><img css={&params.img_css} src={original} /></div> }
                    } else {
                        dom! {
                            <div css={flex_column}>
                                <img css={&params.img_css} src={original} />
                                <label css={btn_css}>
                                    <input
                                        css={hidden_input_css}
                                        type="file"
                                        accept={accept}
                                        on_change_file={make_on_change_file()}
                                    />
                                    {select_label}
                                </label>
                            </div>
                        }
                    }
                }
                None => {
                    if select_label.is_empty() {
                        dom! { <div>{&params.no_image_text}</div> }
                    } else {
                        dom! {
                            <div css={flex_column}>
                                {&params.no_image_text}
                                <label css={btn_css}>
                                    <input
                                        css={hidden_input_css}
                                        type="file"
                                        accept={accept}
                                        on_change_file={make_on_change_file()}
                                    />
                                    {select_label}
                                </label>
                            </div>
                        }
                    }
                }
            },
        }
    });

    let item = item.clone();
    let callback = params.callback.clone();
    let on_dropfile = move |event: DropFileEvent| {
        for file in event.items.into_iter() {
            if let Some(callback) = callback.as_deref() {
                callback(Some(file));
            } else {
                item.set(Some(file));
            }
        }
    };

    let dropzone_css = &params.dropzone_css + &params.dropzone_add_css;

    dom! {
        <div css={dropzone_css} on_dropfile={on_dropfile} {..zone}>
            { image_view }
        </div>
    }
}

#[derive(Clone)]
pub struct DropImageFileParams {
    /// Custom callback when new image dropped, leave empty to automatically set/unset `item`
    pub callback: Option<Rc<dyn Fn(Option<DropFileItem>)>>,
    pub revert_label: String,
    pub cancel_label: String,
    pub no_image_text: String,
    pub dropzone_css: Css,
    pub dropzone_add_css: Css,
    pub img_css: Css,
    /// Label for the "select file" button. Set to empty string to hide the button.
    pub select_label: String,
    /// Value for the `accept` attribute on the hidden file input (e.g. `"image/*"`).
    pub accept: String,
}

impl Default for DropImageFileParams {
    fn default() -> Self {
        Self {
            callback: None,
            revert_label: "Revert".to_string(),
            cancel_label: "Cancel".to_string(),
            no_image_text: "No image".to_string(),
            dropzone_css: css! {"
                width: 400px;
                height: 400px;

                display: flex;
                align-items: center;
                justify-content: center;

                padding: 10px;
            "},
            dropzone_add_css: css! {""},
            img_css: css! {"
                max-width: 100%;
                max-height: 320px;
                object-fit: contain;
            "},
            select_label: "Select file...".to_string(),
            accept: "image/*".to_string(),
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
