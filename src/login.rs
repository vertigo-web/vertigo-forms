use std::rc::Rc;
use vertigo::{
    AttrGroup, Css, DomNode, KeyDownEvent, Resource, Value, bind, bind_rc, component, css, dom,
    transaction,
};

pub type OnSubmit = Rc<dyn Fn(&str, &str)>;

/// Custom renderer for a login input. Receives the value bound to the form so
/// submit/Enter handling keeps working.
pub type RenderInput = Rc<dyn Fn(Value<String>) -> DomNode>;

/// Custom renderer for the submit control. Receives the submit callback to wire up.
pub type RenderSubmit = Rc<dyn Fn(Rc<dyn Fn()>) -> DomNode>;

#[component]
pub fn Login<T: Clone + PartialEq + 'static>(
    on_submit: OnSubmit,
    token_result: Value<Option<Resource<T>>>,
    params: LoginParams,
    /// Attributes passed to both user/pass inputs
    i: AttrGroup,
    /// Attributes passed to submit button
    s: AttrGroup,
) {
    mount_login(on_submit, token_result, params, i, s)
}

pub struct LoginParams {
    pub css: Css,
    pub add_css: Css,
    pub line_css: Css,
    pub line_add_css: Css,
    pub input_css: Css,
    pub submit_css: Css,
    pub submit_add_css: Css,
    pub error_message: Rc<dyn Fn(String) -> String>,
    pub username_label: String,
    pub password_label: String,
    pub button_label: String,
    pub waiting_label: String,
    /// Optional content rendered at the top of the form, before the message line.
    pub header: Option<DomNode>,
    /// Optional content rendered at the bottom of the form, after the submit button.
    pub footer: Option<DomNode>,
    /// Optional custom renderer for the username input. Receives the bound value
    /// so submit/Enter handling keeps working. When `None`, a plain `<input>` is used.
    pub render_username: Option<RenderInput>,
    /// Optional custom renderer for the password input. Receives the bound value.
    /// When `None`, a plain `<input type="password">` is used.
    pub render_password: Option<RenderInput>,
    /// Optional custom renderer for the submit control. Receives the submit callback
    /// to wire up (e.g. `on_click`). When `None`, a plain `<input type="submit">` is used.
    pub render_submit: Option<RenderSubmit>,
}

impl Default for LoginParams {
    fn default() -> Self {
        Self {
            css: css! {"
                width: 250px;
                margin: auto;
                padding: 10px;
                margin-bottom: 10px;
            "},
            add_css: Css::default(),
            line_css: css! {"
                min-height: 1em;
                margin-bottom: 5px;
            "},
            line_add_css: Css::default(),
            input_css: Css::default(),
            submit_css: css! {"
                margin-top: 15px;
            "},
            submit_add_css: Css::default(),
            error_message: Rc::new(|err| err),
            username_label: "Username:".to_string(),
            password_label: "Password:".to_string(),
            button_label: "Login".to_string(),
            waiting_label: "Logging in...".to_string(),
            header: None,
            footer: None,
            render_username: None,
            render_password: None,
            render_submit: None,
        }
    }
}

pub fn mount_login<T: Clone + PartialEq + 'static>(
    on_submit: OnSubmit,
    token_result: Value<Option<Resource<T>>>,
    params: LoginParams,
    input_attrs: AttrGroup,
    submit_attrs: AttrGroup,
) -> DomNode {
    let username = Value::<String>::default();
    let password = Value::<String>::default();

    let submit = bind_rc!(on_submit, username, password, || transaction(|ctx| {
        on_submit(&username.get(ctx), &password.get(ctx));
    }));

    let on_key_down = bind!(submit, |key: KeyDownEvent| {
        if key.code == "Enter" || key.code == "NumpadEnter" {
            submit();
            return true;
        }
        false
    });

    let css = &params.css + &params.add_css;
    let line_css = &params.line_css + &params.line_add_css;
    let submit_css = &params.submit_css + &params.submit_add_css;
    let error_message = params.error_message.clone();
    let waiting_label = params.waiting_label.clone();

    let message_div = bind!(
        line_css,
        token_result.render_value(move |token_result| {
            let css_error = line_css.clone().push_str("color: red;");

            match token_result {
                Some(Resource::Loading) => dom! {
                    <div css={line_css.clone()}>{&waiting_label}</div>
                },
                Some(Resource::Error(err)) => dom! {
                    <div css={css_error}>{error_message(err)}</div>
                },
                _ => dom! {
                    <div css={line_css.clone()} />
                },
            }
        })
    );

    let username_div = match &params.render_username {
        Some(render) => render(username.clone()),
        None => {
            let on_username_change = bind!(username, |new_value: String| username.set(new_value));
            dom! {
                <div css={line_css.clone()}>
                    <div>{&params.username_label}</div>
                    <input
                        css={&params.input_css}
                        value={username.to_computed()}
                        on_input={on_username_change}
                        {..input_attrs.clone()}
                    />
                </div>
            }
        }
    };

    let password_div = match &params.render_password {
        Some(render) => render(password.clone()),
        None => {
            let on_password_change = bind!(password, |new_value| password.set(new_value));
            dom! {
                <div css={line_css}>
                    <div>{&params.password_label}</div>
                    <input
                        css={&params.input_css}
                        value={password.to_computed()}
                        on_input={on_password_change}
                        type="password"
                        {..input_attrs}
                    />
                </div>
            }
        }
    };

    let submit_div = match &params.render_submit {
        Some(render) => render(submit.clone()),
        None => dom! {
            <div css={submit_css}>
                <input
                    type="submit"
                    value={&params.button_label}
                    on_click={move |_| submit()}
                    {..submit_attrs}
                />
            </div>
        },
    };

    let header = params.header;
    let footer = params.footer;

    dom! {
        <div css={css} {on_key_down}>
            { ..header }
            { message_div }
            { username_div }
            { password_div }
            { submit_div }
            { ..footer }
        </div>
    }
}
