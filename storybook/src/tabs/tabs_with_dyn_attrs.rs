use std::rc::Rc;
use vertigo::{DomNode, Value, dom};
use vertigo_forms::{Tab, Tabs};

use crate::bordered_tabs;

#[derive(Clone, PartialEq)]
pub enum MyInnerView {
    InnerView1,
    InnerView2,
}

pub fn tabs_with_dyn_attrs() -> DomNode {
    let current_tab: Value<MyInnerView> = Value::new(MyInnerView::InnerView1);

    let tabs = vec![
        Tab {
            key: MyInnerView::InnerView1,
            name: "Inner View 1".to_string(),
            render: Rc::new(move |curr| {
                dom! {
                    <p>"Inner View 1 content"</p>
                    <p>"Current view: " {curr}</p>
                }
            }),
        },
        Tab {
            key: MyInnerView::InnerView2,
            name: "Inner View 2".to_string(),
            render: Rc::new(move |curr| {
                dom! {
                    <p>"Inner View 2 content"</p>
                    <p>"Current view: " {curr}</p>
                }
            }),
        },
    ];

    dom! {
        <p>
            <Tabs
                current_tab={&current_tab}
                tabs={tabs.clone()}
                params={bordered_tabs()}
                h:style="padding-bottom: 5px"
                c:style="border: 2px solid green"
            />
        </p>
    }
}

impl std::fmt::Display for MyInnerView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InnerView1 => f.write_str("Inner View 1"),
            Self::InnerView2 => f.write_str("Inner View 2"),
        }
    }
}
