use vertigo::{DomNode, dom};

mod data_section;
mod text_field;

pub fn resource_table() -> DomNode {
    dom! {
        <div>
            {text_field::resource_table_text_field()}
            {data_section::resource_table_data_section()}
        </div>
    }
}
