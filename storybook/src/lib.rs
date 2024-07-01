use std::{fmt, rc::Rc};
use vertigo::{css, dom, main, router::Router, DomNode, ToComputed};
use vertigo_forms::{Tab, TabsHeaderParams, TabsHeader, TabsContent};

mod input;
mod multi_select;
mod multi_drop_down;
mod switch;
mod select;
mod select_search;
mod search_panel;
mod drop_image_file;

#[main]
fn render() -> DomNode {
    #[derive(Clone, PartialEq, Eq, Hash)]
    enum TabRoute {
        Input, MultiSelect, MultiDropDown, Switch, Select, SelectSearch, SearchPanel, DropFile,
    }

    impl From<String> for TabRoute {
        fn from(path: String) -> Self {
            match path.as_str() {
                "/input" => Self::Input,
                "/multi_select" => Self::MultiSelect,
                "/multi_drop_down" => Self::MultiDropDown,
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
                Self::MultiSelect => write!(f, "/multi_select"),
                Self::MultiDropDown => write!(f, "/multi_drop_down"),
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
            key: TabRoute::MultiSelect,
            name: "MultiSelect".to_string(),
            render: Rc::new(multi_select::multi_select)
        },

        Tab {
            key: TabRoute::MultiDropDown,
            name: "MultiDropDown".to_string(),
            render: Rc::new(multi_drop_down::multi_drop_down)
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

    let current_tab = Router::<TabRoute>::new_history_router();

    let css_tab_content = css!("
        border: solid 1px black;
        padding: 0px 10px;
        ");

    let header = TabsHeader::<Router<TabRoute>, _> {
        current_tab: current_tab.clone(),
        tabs: tabs.clone(),
        params: TabsHeaderParams {
            header_item_add_css: css!("
                border: 1px solid black;
                padding: 0px 10px;
            "),
            header_active_item_add_css: css!("
                background-color: lightgray;
            "),
            ..Default::default()
        }
    }.mount();

    dom! {
        <html>
            <head>
                <title>"Vertigo Forms Storybook"</title>
            </head>
            <body>
                {header}
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
