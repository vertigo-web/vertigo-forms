use std::rc::Rc;
use vertigo::{AttrGroup, AutoMap, DomNode, Resource, ToComputed, Value, bind, component, dom};

pub trait SearchResult {
    fn is_empty(&self) -> bool;
}

impl<T> SearchResult for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Clone)]
pub struct SearchPanelParams {
    pub min_chars: usize,
    pub prompt: String,
    pub hint: String,
    pub loading_text: String,
    pub empty_text: String,
}

impl Default for SearchPanelParams {
    fn default() -> Self {
        Self {
            min_chars: 3,
            prompt: "Search: ".to_string(),
            hint: "Enter at least {min_chars} letters".to_string(),
            loading_text: "Loading...".to_string(),
            empty_text: "No results".to_string(),
        }
    }
}

/// Component that takes query and loads/computes a result.
#[component]
pub fn SearchPanel<T, K>(
    query: Value<String>,
    cache: AutoMap<String, K>,
    render_results: Rc<dyn Fn(Rc<T>) -> DomNode>,
    params: SearchPanelParams,
    /// Add attributes to the container
    c: AttrGroup,
    /// Add attributes to the input
    i: AttrGroup,
) where
    T: SearchResult + PartialEq + Clone + 'static,
    K: ToComputed<Resource<Rc<T>>> + Clone + 'static,
{
    let prompt = params.prompt.clone();
    let content = query.render_value(move |query| {
        let SearchPanelParams {
            min_chars,
            prompt: _,
            hint,
            loading_text,
            empty_text,
        } = params.clone();
        if query.len() < min_chars {
            let msg = hint.replace("{min_chars}", &min_chars.to_string());
            return dom! { <div>{msg}</div> };
        }
        let render_results = render_results.clone();
        let content = cache
            .get(&query)
            .to_computed()
            .render_value(move |books| match books {
                Resource::Loading => dom! { <div>{loading_text.clone()}</div> },
                Resource::Ready(dataset) => {
                    if !dataset.is_empty() {
                        render_results(dataset)
                    } else {
                        dom! {
                            <div>{empty_text.clone()}</div>
                        }
                    }
                }
                Resource::Error(err) => {
                    dom! { <div>{err}</div> }
                }
            });
        dom! { <div>{content}</div> }
    });

    let on_input = bind!(query, |new_value: String| {
        query.set(new_value);
    });

    let value = query.to_computed();

    dom! {
        <div {..c}>
            { prompt }
            <input {value} on_input={on_input} {..i} />
            { content }
        </div>
    }
}
