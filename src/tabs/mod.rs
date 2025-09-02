use std::rc::Rc;
use vertigo::{Computed, Css, DomElement, DomNode, Reactive, ToComputed, bind, css, dom};

#[derive(Clone)]
pub struct Tab<K> {
    pub key: K,
    pub name: String,
    pub render: Rc<dyn Fn(&K) -> DomNode>,
}

#[derive(Clone)]
pub struct TabsParams {
    pub header_css: Css,
    pub header_item_css: Css,
    pub header_item_add_css: Css,
    pub header_active_item_add_css: Css,
    pub content_css: Css,
    pub container_css: Css,
}

impl Default for TabsParams {
    fn default() -> Self {
        Self {
            // render_header_item: None,
            header_css: css! {"
                display: flex;
                flex-wrap: wrap;
                gap: 10px;
                margin: 0px;
                padding: 0px;
            "},
            header_item_css: css! {"
                cursor: pointer;
            "},
            header_item_add_css: Css::default(),
            header_active_item_add_css: Css::default(),
            content_css: Css::default(),
            container_css: Css::default(),
        }
    }
}

/// [TabsHeader] and [TabsContent] rendered next to each other.
pub struct Tabs<R: Reactive<K>, K: Clone> {
    pub current_tab: R,
    pub tabs: Vec<Tab<K>>,
    pub params: TabsParams,
}

impl<R, K> Tabs<R, K>
where
    R: Reactive<K> + ToComputed<K> + Clone + 'static,
    K: Clone + PartialEq + 'static,
{
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(self) -> DomNode {
        let Self {
            current_tab,
            tabs,
            params,
        } = self;

        let current_computed = current_tab.to_computed();

        dom! {
            <div css={&params.container_css}>
                <TabsHeader
                    {&current_tab}
                    tabs={tabs.clone()}
                    params={params.clone()}
                />
                <TabsContent
                    current_tab={current_computed}
                    tabs={tabs}
                    {params}
                />
            </div>
        }
    }
}

/// Nagivation bar for [TabsContent].
pub struct TabsHeader<R: Reactive<K>, K: Clone> {
    pub current_tab: R,
    pub tabs: Vec<Tab<K>>,
    pub params: TabsParams,
}

impl<R, K> TabsHeader<R, K>
where
    R: Reactive<K> + ToComputed<K> + Clone + 'static,
    K: Clone + PartialEq + 'static,
{
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(self) -> DomNode {
        let Self {
            current_tab,
            tabs,
            params,
        } = self;

        let header_item_css = params.header_item_css + params.header_item_add_css;
        let header_active_item_add_css = params.header_active_item_add_css;

        // let current_tab_clone = current_tab.clone();
        current_tab
            .to_computed()
            .render_value(move |current_tab_val| {
                let header = DomElement::new("ul").css(params.header_css.clone());

                tabs.iter().for_each(|tab| {
                    let on_click = bind!(current_tab, tab | _ | current_tab.set(tab.key.clone()));
                    let header_item_css = if current_tab_val == tab.key {
                        &header_item_css + &header_active_item_add_css
                    } else {
                        header_item_css.clone()
                    };
                    let item_css = css!("display: block;");
                    header.add_child(dom! {
                        <li css={item_css}>
                            <a  css={header_item_css} on_click={on_click}>{&tab.name}</a>
                        </li>
                    });
                });

                header.into()
            })
    }
}

/// Renders content controlled by [TabsHeader] bar.
pub struct TabsContent<K: Clone> {
    pub current_tab: Computed<K>,
    pub tabs: Vec<Tab<K>>,
    pub params: TabsParams,
}

impl<K> TabsContent<K>
where
    K: Clone + PartialEq + 'static,
{
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(self) -> DomNode {
        let Self {
            current_tab,
            tabs,
            params,
        } = self;

        current_tab.render_value(move |current_tab| {
            render_tab_content(&current_tab, &current_tab, &tabs, &params)
        })
    }
}

/// Renders content controlled by [TabsHeader] bar,
/// but allows to map groups of possible values to single tab,
/// handy when using [Tabs] component connected with routing
pub struct TabsContentMapped<K: Clone> {
    pub current_tab: Computed<K>,
    pub tabs: Vec<Tab<K>>,
    pub tab_map: Rc<dyn Fn(K) -> K>,
    pub params: TabsParams,
}

impl<K> TabsContentMapped<K>
where
    K: Clone + PartialEq + 'static,
{
    pub fn into_component(self) -> Self {
        self
    }

    pub fn mount(self) -> DomNode {
        let Self {
            current_tab,
            tabs,
            tab_map,
            params,
        } = self;

        current_tab.render_value(move |current_tab| {
            render_tab_content(&current_tab, &tab_map(current_tab.clone()), &tabs, &params)
        })
    }
}

fn render_tab_content<K: PartialEq + Clone>(
    current_tab: &K,
    effective_tab: &K,
    tabs: &[Tab<K>],
    params: &TabsParams,
) -> DomNode {
    let inner = match tabs.iter().find(|tab| &tab.key == effective_tab).cloned() {
        Some(tab) => (tab.render)(current_tab),
        _ => dom! { <p>"Non-existent tab set"</p> },
    };

    dom! {
        <div css={params.content_css.clone()}>
            {inner}
        </div>
    }
}
