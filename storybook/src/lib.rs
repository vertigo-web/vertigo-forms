use std::{fmt, rc::Rc};
use vertigo::{css, dom, main, router::Router, DomNode};
use vertigo_forms::{Tab, Tabs, TabsParams};

mod drop_image_file;
mod form;
mod input;
mod login;
mod multi_drop_down;
mod multi_select;
mod popup;
mod search_panel;
mod select;
mod select_search;
mod switch;
mod tabs;
mod with_loader;

#[main]
fn render() -> DomNode {
    #[derive(Clone, PartialEq, Eq, Hash)]
    enum TabRoute {
        Input,
        Popup,
        MultiSelect,
        MultiDropDown,
        Switch,
        Select,
        SelectSearch,
        SearchPanel,
        Form,
        Tabs,
        DropFile,
        Login,
        Spinner,
        WithLoader,
    }

    impl From<String> for TabRoute {
        fn from(path: String) -> Self {
            match path.as_str() {
                "/input" => Self::Input,
                "/popup" => Self::Popup,
                "/multi_select" => Self::MultiSelect,
                "/multi_drop_down" => Self::MultiDropDown,
                "/switch" => Self::Switch,
                "/select" => Self::Select,
                "/select_search" => Self::SelectSearch,
                "/search_panel" => Self::SearchPanel,
                "/form" => Self::Form,
                "/tabs" => Self::Tabs,
                "/drop_file" => Self::DropFile,
                "/login" => Self::Login,
                "/spinner" => Self::Spinner,
                "/with_loader" => Self::WithLoader,
                _ => Self::Input,
            }
        }
    }

    impl fmt::Display for TabRoute {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Input => write!(f, "/input"),
                Self::Popup => write!(f, "/popup"),
                Self::MultiSelect => write!(f, "/multi_select"),
                Self::MultiDropDown => write!(f, "/multi_drop_down"),
                Self::Switch => write!(f, "/switch"),
                Self::Select => write!(f, "/select"),
                Self::SelectSearch => write!(f, "/select_search"),
                Self::SearchPanel => write!(f, "/search_panel"),
                Self::Form => write!(f, "/form"),
                Self::Tabs => write!(f, "/tabs"),
                Self::DropFile => write!(f, "/drop_file"),
                Self::Login => write!(f, "/login"),
                Self::Spinner => write!(f, "/spinner"),
                Self::WithLoader => write!(f, "/with_loader"),
            }
        }
    }

    let tabs = vec![
        Tab {
            key: TabRoute::Input,
            name: "Input".to_string(),
            render: Rc::new(|_| input::input()),
        },
        Tab {
            key: TabRoute::Popup,
            name: "Popup".to_string(),
            render: Rc::new(|_| popup::popup()),
        },
        Tab {
            key: TabRoute::Switch,
            name: "Switch".to_string(),
            render: Rc::new(|_| switch::switch()),
        },
        Tab {
            key: TabRoute::Select,
            name: "Select".to_string(),
            render: Rc::new(|_| select::select()),
        },
        Tab {
            key: TabRoute::MultiSelect,
            name: "MultiSelect".to_string(),
            render: Rc::new(|_| multi_select::multi_select()),
        },
        Tab {
            key: TabRoute::MultiDropDown,
            name: "MultiDropDown".to_string(),
            render: Rc::new(|_| multi_drop_down::multi_drop_down()),
        },
        Tab {
            key: TabRoute::SelectSearch,
            name: "Select/Search".to_string(),
            render: Rc::new(|_| select_search::select_search()),
        },
        Tab {
            key: TabRoute::SearchPanel,
            name: "Search Panel".to_string(),
            render: Rc::new(|_| search_panel::search_panel()),
        },
        Tab {
            key: TabRoute::Form,
            name: "Form".to_string(),
            render: Rc::new(|_| form::form()),
        },
        Tab {
            key: TabRoute::Tabs,
            name: "Tabs".to_string(),
            render: Rc::new(|_| tabs::tabs()),
        },
        Tab {
            key: TabRoute::DropFile,
            name: "Drop Image File".to_string(),
            render: Rc::new(|_| drop_image_file::drop_file()),
        },
        Tab {
            key: TabRoute::Login,
            name: "Login".to_string(),
            render: Rc::new(|_| login::login()),
        },
        Tab {
            key: TabRoute::Spinner,
            name: "Spinner".to_string(),
            render: Rc::new(|_| dom! { <vertigo_forms::Spinner /> }),
        },
        Tab {
            key: TabRoute::WithLoader,
            name: "With Loader".to_string(),
            render: Rc::new(|_| with_loader::with_loader()),
        },
    ];

    let current_tab = Router::<TabRoute>::new_history_router();

    dom! {
        <html>
            <head>
                <title>"Vertigo Forms Storybook"</title>
            </head>
            <body>
                    <Tabs
                        {&current_tab}
                        {tabs}
                        params={bordered_tabs()}
                    />
            </body>
        </html>
    }
}

pub fn bordered_tabs() -> TabsParams {
    TabsParams {
        header_item_add_css: css! {"
            border: 1px solid black;
            padding: 0px 10px;
        "},
        header_active_item_add_css: css! {"
            background-color: lightgray;
        "},
        content_css: css! {"
            border: solid 1px black;
            padding: 5px 10px;
        "},
        container_css: css! {"
            margin: 10px;
        "},
        ..Default::default()
    }
}
