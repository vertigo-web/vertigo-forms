use std::rc::Rc;
use vertigo::{dom, DomNode, Value};
use vertigo_forms::{Tab, Tabs};

use crate::bordered_tabs;

mod form1;

mod form2;

mod tabbed_form;

pub fn form() -> DomNode {
    type MyTabs<T> = Tabs<Value<T>, T>;
    dom! {
        <MyTabs
            current_tab={Value::new("form1")}
            tabs={vec![
                Tab {
                    key: "form1",
                    name: "Form 1".to_string(),
                    render: Rc::new(move |_| {
                        dom! { <form1::Form1 /> }
                    }),
                },
                Tab {
                    key: "form2",
                    name: "Form 2".to_string(),
                    render: Rc::new(move |_| {
                        dom! { <form2::Form2 /> }
                    }),
                },
                Tab {
                    key: "tabbed_form",
                    name: "Tabbed Form".to_string(),
                    render: Rc::new(move |_| {
                        dom! { <tabbed_form::TabbedForm /> }
                    }),
                },
            ]}
            params={bordered_tabs()}
        />
    }
}
