use std::rc::Rc;

use vertigo::{DomNode, Value, bind, css, dom};
use vertigo_forms::{Modal, ModalParams, RenderModalContent};

pub fn modal() -> DomNode {
    let cat = Value::new(false);
    let show_cat = bind!(cat, |_| cat.set(true));
    // A field and a button, so the keyboard has where to go, and a name typed in is gone when
    // the window opens again
    let cat_content: RenderModalContent = Rc::new(|| {
        dom! {
            <div>
                <p>"Clearly not a dog."</p>
                <label>"Name " <input name="name" /></label>
                " "
                <button type="button">"Pet"</button>
            </div>
        }
    });

    let list = Value::new(false);
    let show_list = bind!(list, |_| list.set(true));
    let list_content: RenderModalContent = Rc::new(|| {
        let cats = (1..=60).map(|number| dom! { <li>"Cat " {number}</li> });
        dom! { <ol>{..cats}</ol> }
    });
    let wide = ModalParams {
        window_css: css! {"
            width: 100%;
            height: 100%;
            background-color: ivory;
        "},
        close_label: "Close the list".to_string(),
        ..Default::default()
    };

    dom! {
        <div>
            <p>
                <button type="button" on_click={show_cat}>"Show the cat"</button>
                " "
                <button type="button" on_click={show_list}>"Show all the cats"</button>
            </p>
            <Modal open={cat} title="The cat" content={cat_content} params={} />
            <Modal open={list} title="All the cats" content={list_content} params={wide} w:id="cats" />
        </div>
    }
}
