use std::rc::Rc;

use vertigo::{bind_rc, dom, DomNode, Value};
use vertigo_forms::{Tab, TabsContentMapped, TabsHeader};

#[derive(Clone, PartialEq)]
pub enum MyView {
    View1,
    View1SubView1,
    View1SubView2,
    View2SubView1,
    View2SubView2,
}

pub fn tabs() -> DomNode {
    let current_tab: Value<MyView> = Value::new(MyView::View1);

    let change_subtab_1 = bind_rc!(current_tab, |new_tab: MyView| {
        let current_tab = current_tab.clone();
        move |_| current_tab.set(new_tab.clone())
    });
    let change_subtab_2 = change_subtab_1.clone();

    let tabs = vec![
        Tab {
            key: MyView::View1,
            name: "View 1".to_string(),
            render: Rc::new(move |curr| {
                dom! {
                    <p>"View 1 content"</p>
                    <p>"Current view: " {curr}</p>
                    <button on_click={change_subtab_1(MyView::View1SubView1)}>"Go to sub view 1.1"</button>
                    <button on_click={change_subtab_1(MyView::View1SubView2)}>"Go to sub view 1.2"</button>
                }
            }),
        },
        Tab {
            key: MyView::View2SubView1,
            name: "View 2".to_string(),
            render: Rc::new(move |curr| {
                dom! {
                    <p>"View 2 content"</p>
                    <p>"Current view: " {curr}</p>
                    <button on_click={change_subtab_2(MyView::View2SubView1)}>"Go to sub view 2.1"</button>
                    <button on_click={change_subtab_2(MyView::View2SubView2)}>"Go to sub view 2.2"</button>
                    <button on_click={change_subtab_2(MyView::View1)}>"Go back to view 1"</button>
                }
            }),
        },
    ];

    dom! {
        <p>
            <TabsHeader
                current_tab={&current_tab}
                tabs={tabs.clone()}
                params={}
            />
            <TabsContentMapped
                current_tab={current_tab.to_computed()}
                {tabs}
                tab_map={&Rc::new(|view|
                    match view {
                        MyView::View1 | MyView::View1SubView1 | MyView::View1SubView2 => MyView::View1,
                        MyView::View2SubView1 | MyView::View2SubView2 => MyView::View2SubView1,
                    }
                )}
                params={}
            />
        </p>
    }
}

impl std::fmt::Display for MyView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::View1 => f.write_str("View1"),
            Self::View1SubView1 => f.write_str("View1SubView1"),
            Self::View1SubView2 => f.write_str("View1SubView2"),
            Self::View2SubView1 => f.write_str("View2SubView1"),
            Self::View2SubView2 => f.write_str("View2SubView2"),
        }
    }
}
