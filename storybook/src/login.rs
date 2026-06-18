use std::rc::Rc;
use vertigo::{DomNode, Resource, Value, bind, bind_rc, css, dom};
use vertigo_forms::Input;
use vertigo_forms::login::{Login, LoginParams, RenderInput, RenderSubmit};

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

            <hr />

            <Login {on_submit} {token_result} params={LoginParams {
                username_label: "E-Mail".to_string(),
                ..Default::default()
            }} />

            {result_html}

            <hr />

            { customized() }
        </div>
    }
}

/// Demonstrates the optional render-customization slots on `LoginParams`:
/// `header`, `footer`, `render_username`, `render_password` and `render_submit`.
fn customized() -> DomNode {
    let token_result = Value::default();
    let on_submit = bind_rc!(token_result, |user: &str, pass: &str| {
        if user == "test" && pass == "123" {
            token_result.set(Some(Resource::Ready("qwerty1234".to_string())));
        } else {
            token_result.set(Some(Resource::Error(
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

    let label_css = css! {"
        display: block;
        font-size: 0.85em;
        margin-bottom: 3px;
    "};

    // Reuse the existing `Input` component for the username slot.
    let render_username: RenderInput = {
        let label_css = label_css.clone();
        Rc::new(move |value: Value<String>| {
            dom! {
                <div>
                    <label css={&label_css}>"👤 Username"</label>
                    <Input {value} />
                </div>
            }
        })
    };

    // Hand-rolled input wiring for the password slot.
    let render_password: RenderInput = {
        let label_css = label_css.clone();
        Rc::new(move |value: Value<String>| {
            let on_input = bind!(value, |new_value: String| value.set(new_value));
            dom! {
                <div>
                    <label css={&label_css}>"🔒 Password"</label>
                    <input
                        type="password"
                        value={value.to_computed()}
                        {on_input}
                    />
                </div>
            }
        })
    };

    // Custom submit control wired to the form's submit callback.
    let render_submit: RenderSubmit = Rc::new(|submit: Rc<dyn Fn()>| {
        dom! {
            <div css={css! {"margin-top: 15px;"}}>
                <button
                    css={css! {"
                        padding: 6px 16px;
                        cursor: pointer;
                    "}}
                    on_click={move |_| submit()}
                >
                    "Sign in →"
                </button>
            </div>
        }
    });

    let params = LoginParams {
        header: Some(dom! { <h3>"Custom login form"</h3> }),
        footer: Some(dom! {
            <p css={css! {"font-size: 0.85em; margin-top: 10px;"}}>"Forgot your password?"</p>
        }),
        render_username: Some(render_username),
        render_password: Some(render_password),
        render_submit: Some(render_submit),
        ..Default::default()
    };

    dom! {
        <div>
            <Login {on_submit} {token_result} {params} />
            {result_html}
        </div>
    }
}
