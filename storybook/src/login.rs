use vertigo::{DomNode, Resource, Value, bind_rc, dom};
use vertigo_forms::login::{Login, LoginParams};

pub fn login() -> DomNode {
    let outcome_text = Value::new(String::new());
    let token_result = Value::default();
    let on_submit = bind_rc!(token_result, |user: &str, pass: &str| {
        outcome_text.set(["Tried to login as user ", user, " and password ", pass].concat());
        if user == "test" && pass == "123" {
            token_result.set(Some(vertigo::Resource::Ready("qwerty1234".to_string())));
        } else {
            token_result.set(Some(vertigo::Resource::Error(
                "Invalid password, try test/123".to_string(),
            )));
        }
    });

    let result_html = token_result.render_value_option(|v| {
        v.map(|v| match v {
            Resource::Loading => dom! { <p>"Logging..."</p> },
            Resource::Ready(token) => dom! { <p>"Login successful, token " {token}</p> },
            Resource::Error(err) => dom! { <p>"Login error: " {err}</p> },
        })
    });

    dom! {
        <div>
            <Login on_submit={on_submit.clone()} token_result={token_result.clone()} params={} />

            <Login {on_submit} {token_result} params={LoginParams {
                username_label: "E-Mail".to_string(),
                ..Default::default()
            }} />

            {result_html}
        </div>
    }
}
