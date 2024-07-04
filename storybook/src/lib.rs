use std::{fmt, rc::Rc};
use vertigo::{css, dom, main, router::Router, Computed, Context, DomNode, ToComputed, Value};
use vertigo_forms::{CurrentTab, Tab, TabsContent, TabsHeader, TabsHeaderParams};

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

    impl From<String> for TabRoute {
        fn from(path: String) -> Self {
            match path.as_str() {
                "/input" => Self::Input,
                "/switch" => Self::Switch,
                "/select" => Self::Select,
                "/select_search" => Self::SelectSearch,
                "/search_panel" => Self::SearchPanel,
                "/drop_file" => Self::DropFile,
                _ => Self::Input,
            }
        }
    }

    impl fmt::Display for TabRoute {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Input => write!(f, "/input"),
                Self::Switch => write!(f, "/switch"),
                Self::Select => write!(f, "/select"),
                Self::SelectSearch => write!(f, "/select_search"),
                Self::SearchPanel => write!(f, "/search_panel"),
                Self::DropFile => write!(f, "/drop_file"),
            }
        }
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

    #[derive(PartialEq, Clone)]
    pub struct MyCurrentTab(pub Router::<TabRoute>);

    impl ToComputed<TabRoute> for MyCurrentTab {
        fn to_computed(&self) -> Computed<TabRoute> {
            self.0.route.to_computed()
        }
    }

    impl CurrentTab<TabRoute> for MyCurrentTab {
        fn set(&self, value: TabRoute) {
            Router::set(&self.0, value)
        }

        fn get(&self, context: &Context) -> TabRoute {
            self.0.route.get(context)
        }
    }

    let current_tab = MyCurrentTab(Router::<TabRoute>::new_history_router());

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
                        current_tab={current_tab.to_computed()}
                        tabs={tabs}
                    />
                </div>
            </body>
        </html>
    }
}
