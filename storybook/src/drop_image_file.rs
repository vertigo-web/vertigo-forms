use std::rc::Rc;
use vertigo::{css, dom, DomNode, DropFileItem, Value};
use vertigo_forms::{image_as_uri, DropImageFile, DropImageFileParams};

pub fn drop_file() -> DomNode {
    let value = Value::new(Some(Rc::new("https://picsum.photos/200".to_string()))).to_computed();
    let image = Value::default();

    let image_element = image
        .map(|image: Option<DropFileItem>| image.map(|image| image_as_uri(&image)))
        .render_value_option(|image_data| {
            image_data.map(|image_data| dom! { <img src={image_data} /> })
        });

    dom! {
        <p>
            <DropImageFile
                original_link={value.clone()}
                item={image}
                params={DropImageFileParams {
                    img_css: css! { "
                        max-width: 400px;
                        max-height: 400px;
                    "},
                    ..Default::default()
                }}
            />
        </p>
        <p>"Dropped image: "</p>
        <p>{image_element}</p>
    }
}
