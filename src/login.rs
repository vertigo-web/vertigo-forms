use std::rc::Rc;
use vertigo::{bind, bind_rc, css, dom, transaction, Css, DomNode, KeyDownEvent, Resource, Value};

pub type OnSubmit = Rc<dyn Fn(&str, &str)>;

pub struct Login<T> {
    pub on_submit: OnSubmit,
    pub token_result: Value<Option<Resource<T>>>,
    pub params: LoginParams,
}

pub struct LoginParams {
    pub css: Css,
    pub add_css: Css,
    pub line_css: Css,
    pub line_add_css: Css,
    pub submit_css: Css,
    pub submit_add_css: Css,
    pub error_message: Rc<dyn Fn(String) -> String>,
}

impl Default for LoginParams {
    fn default() -> Self {
        Self {
            css: css!("
                width: 250px;
                margin: auto;
                padding: 10px;
                margin-bottom: 10px;
            "),
            add_css: Css::default(),
            line_css: css!("
                min-height: 1em;
                margin-bottom: 5px;
            "),
            line_add_css: Css::default(),
            submit_css: css!("
                margin-top: 15px;
            "),
            submit_add_css: Css::default(),
            error_message: Rc::new(|err| err),
        }
    }
}

impl<T: Clone + PartialEq + 'static> Login<T> {
    pub fn mount(&self) -> DomNode {
        let Self { on_submit, token_result, params } = self;

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

        let css = params.css.clone().extend(params.add_css.clone());
        let line_css = params.line_css.clone().extend(params.line_add_css.clone());
        let submit_css = params.submit_css.clone().extend(params.submit_add_css.clone());
        let error_message = params.error_message.clone();

        let message_div = bind!(line_css, token_result.render_value(move |token_result| {
            let css_error = line_css.clone().push_str("
                color: red;
            ");

            match token_result {
                Some(Resource::Loading) => dom! {
                    <div css={line_css.clone()}>"Logging in..."</div>
                },
                Some(Resource::Error(err)) => dom! {
                    <div css={css_error}>{error_message(err)}</div>
                },
                _ => dom! {
                    <div css={line_css.clone()} />
                },
            }
        }));

        let on_username_change = bind!(username, |new_value: String|
            username.set(new_value)
        );

        let username_div = dom! {
            <div css={line_css.clone()}>
                <div>"Username:"</div>
                <input value={username.to_computed()} on_input={on_username_change} />
            </div>
        };

        let on_password_change = bind!(password, |new_value|
            password.set(new_value)
        );

        let password_div = dom! {
            <div css={line_css}>
                <div>"Password:"</div>
                <input value={password.to_computed()} on_input={on_password_change} type="password" />
            </div>
        };

        dom! {
            <div css={css} {on_key_down}>
                { message_div }
                { username_div }
                { password_div }
                <div css={submit_css}>
                    <input type="submit" value="Login" on_click={move || submit()} />
                </div>
            </div>
        }
    }
}
