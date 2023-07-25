use std::rc::Rc;
use vertigo::{css, main, Value, DomNode, dom};
use vertigo_forms::{Tab, TabsHeaderParams, TabsHeader, TabsContent};

mod input;
mod switch;
mod select;
mod select_search;
mod search_panel;
mod drop_image_file;

#[main]
fn render() -> DomNode {
    #[derive(Clone, PartialEq, Eq, Hash)]
    enum TabRoute {
        Input, Switch, Select, SelectSearch, SearchPanel, DropFile,
    }

    let tabs = vec![
        Tab {
            key: TabRoute::Input,
            name: "Input".to_string(),
            render: Rc::new(input::input)
        },

        Tab {
            key: TabRoute::Switch,
            name: "Switch".to_string(),
            render: Rc::new(switch::switch)
        },

        Tab {
            key: TabRoute::Select,
            name: "Select".to_string(),
            render: Rc::new(select::select)
        },

        Tab {
            key: TabRoute::SelectSearch,
            name: "Select/Search".to_string(),
            render: Rc::new(select_search::select_search)
        },

        Tab {
            key: TabRoute::SearchPanel,
            name: "Search Panel".to_string(),
            render: Rc::new(search_panel::search_panel)
        },

        Tab {
            key: TabRoute::DropFile,
            name: "Drop Image File".to_string(),
            render: Rc::new(drop_image_file::drop_file)
        },
    ];

    let current_tab = Value::new(TabRoute::Switch);

    let css_tab_content = css!("
        border: solid 1px black;
        padding: 0px 10px;
        ");

    dom! {
        <html>
            <head />
            <body>
                <TabsHeader
                    current_tab={&current_tab}
                    tabs={&tabs}
                    params={TabsHeaderParams {
                        header_item_add_css: css!("
                            border: 1px solid black;
                            padding: 0px 10px;
                        "),
                        header_active_item_add_css: css!("
                            background-color: lightgray;
                        "),
                        ..Default::default()
                    }}
                />
                <div css={css_tab_content}>
                    <TabsContent
                        current_tab={current_tab}
                        tabs={tabs}
                    />
                </div>
            </body>
        </html>
    }
}
