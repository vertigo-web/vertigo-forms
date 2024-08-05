use vertigo::{dom, DomNode, DropFileItem, Value};
use vertigo_forms::{image_as_uri, DropImageFile};

pub fn drop_file() -> DomNode {
    let value = Value::new(Some("https://picsum.photos/200".to_string())).to_computed();
    let image = Value::default();

    let image_data =
        image.map(|image: Option<DropFileItem>| image.map(|image| image_as_uri(&image)));

    dom! {
        <p>
            <DropImageFile
                original_link={value.clone()}
                item={image}
                params={}
            />
        </p>
        <p>"Dropped image: "</p>
        <p><img src={image_data}/></p>
    }
}
