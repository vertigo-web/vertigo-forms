use std::rc::Rc;
use vertigo::{DomNode, dom, Value, AutoMap, Computed, Resource};
use vertigo_forms::{SearchPanel, SearchPanelParams};

pub fn search_panel() -> DomNode {
    let query = Value::default();

    // AutoMap takes query and mocks server response by doing some computation,
    // this is counting the words
    let cache: AutoMap<String, Computed<Resource<Vec<String>>>> = AutoMap::new(
        |_, query: &String| {
            let query = query.clone();
            Computed::from(move |_| Resource::Ready(
                query.split_whitespace().map(|s| s.to_string()).collect()
            ))
        }
    );

    let render_results: Rc<dyn Fn(Rc<Vec<String>>) -> DomNode> = Rc::new(|result| {
        let count = result.len();
        dom! { <p>"Word count: " {count}</p> }
    });

    dom! {
        <p>
            <SearchPanel
                query={query}
                cache={cache}
                render_results={render_results}
                params={SearchPanelParams {
                    prompt: "Enter words: ".to_string(),
                    min_chars: 0,
                    ..Default::default()
                }}
            />
        </p>
    }
}
