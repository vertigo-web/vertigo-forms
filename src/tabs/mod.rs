use std::rc::Rc;
use vertigo::{bind, css, dom, Computed, Context, Css, DomElement, DomNode, ToComputed, Value};

pub trait CurrentTab<T>: PartialEq {
    fn set(&self, value: T);
    fn get(&self, context: &Context) -> T;
}

impl<T> CurrentTab<T> for Value<T>
where
    T: Clone + PartialEq + 'static
{
    fn set(&self, value: T) {
        Value::set(self, value)
    }

    fn get(&self, context: &Context) -> T {
        Value::get(self, context)
    }
}

#[derive(Clone)]
pub struct Tab<K> {
    pub key: K,
    pub name: String,
    pub render: Rc<dyn Fn() -> DomNode>,
}

pub type RenderHeaderFunc<K> = Rc<dyn Fn(&Tab<K>) -> DomNode>;

pub struct TabsHeaderParams<K> {
    pub render_header_item: Option<RenderHeaderFunc<K>>,
    pub header_css: Css,
    pub header_item_css: Css,
    pub header_item_add_css: Css,
    pub header_active_item_add_css: Css,
}

impl<K> Default for TabsHeaderParams<K> {
    fn default() -> Self {
        Self {
            render_header_item: None,
            header_css: css!("
                display: flex;
                gap: 10px;
                margin: 0px;
                padding: 0px;
            "),
            header_item_css: css!("
                cursor: pointer;
            "),
            header_item_add_css: css!(""),
            header_active_item_add_css: css!(""),
        }
    }
}

/// `TabsHeader` and `TabsContent` rendered next to each other.
pub struct Tabs<R: CurrentTab<K>, K: Clone> {
    pub current_tab: R,
    pub tabs: Vec<Tab<K>>,
    pub params: TabsHeaderParams<K>,
}

impl<R, K> Tabs<R, K>
where
    R: CurrentTab<K> + ToComputed<K> + Clone + 'static,
    K: Clone + PartialEq + 'static,
{
    pub fn mount(self) -> DomNode {
        let Self { current_tab, tabs, params } = self;

        let current_computed = current_tab.to_computed();

        let header = TabsHeader::<R, K> {
            current_tab,
            tabs: tabs.clone(),
            params,
        }.mount();

        dom! {
            <div>
                {header}
                <TabsContent
                    current_tab={current_computed}
                    tabs={tabs}
                />
            </div>
        }
    }
}

/// Nagivation bar for TabContent.
pub struct TabsHeader<R: CurrentTab<K>, K: Clone> {
    pub current_tab: R,
    pub tabs: Vec<Tab<K>>,
    pub params: TabsHeaderParams<K>,
}

impl<R, K> TabsHeader<R, K>
where
    R: CurrentTab<K> + ToComputed<K> + Clone + 'static,
    K: Clone + PartialEq + 'static,
{
    pub fn mount(self) -> DomNode {
        let Self { current_tab, tabs, params } = self;

        let header_item_css = params.header_item_css.extend(params.header_item_add_css);
        let header_active_item_add_css = params.header_active_item_add_css;

        // let current_tab_clone = current_tab.clone();
        current_tab.to_computed().render_value(move |current_tab_val| {
            let header = DomElement::new("ul")
                .css(params.header_css.clone());

            tabs.iter().for_each(|tab| {
                if let Some(render_header_item) = &params.render_header_item {
                    // Custom item rendering
                    header.add_child(render_header_item(tab));
                } else {
                    // Default item rendering
                    let on_click = bind!(current_tab, tab
                        || current_tab.set(tab.key.clone())
                    );
                    let header_item_css = if current_tab_val == tab.key {
                        header_item_css.clone().extend(header_active_item_add_css.clone())
                    } else {
                        header_item_css.clone()
                    };
                    let item_css = css!("display: block;");
                    header.add_child(
                        dom! {
                            <li css={item_css}>
                                <a  css={header_item_css} on_click={on_click}>{&tab.name}</a>
                            </li>
                        }
                    );
                }
            });

            header.into()
        })
    }
}

/// Renders content controlled by TabsHeader bar.
pub struct TabsContent<K: Clone> {
    pub current_tab: Computed<K>,
    pub tabs: Vec<Tab<K>>,
}

impl<K> TabsContent<K>
where
    K: Clone + PartialEq + 'static,
{
    pub fn mount(self) -> DomNode {
        let Self { current_tab, tabs} = self;

        current_tab.render_value(move |current_tab| {
            match tabs.iter().find(|tab| tab.key == current_tab).cloned() {
                Some(tab) => (tab.render)(),
                _ => dom! { <p>"Non-existent tab set"</p> }
            }
        })
    }
}
