use std::rc::Rc;
use vertigo::{component, css, dom};

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonColor {
    Primary,
    Success,
    Danger,
    Secondary,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Text,
    Outline,
}

#[component]
pub fn Button(
    label: String,
    on_click: Rc<dyn Fn() + 'static>,
    color: ButtonColor,
    variant: ButtonVariant,
) {
    let style = match (variant, color) {
        (ButtonVariant::Text, ButtonColor::Primary) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 600;
            color: #007bff;
            :hover { text-decoration: underline; }
        "},
        (ButtonVariant::Text, ButtonColor::Success) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 600;
            color: #28a745;
            :hover { text-decoration: underline; }
        "},
        (ButtonVariant::Text, ButtonColor::Danger) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 600;
            color: #dc3545;
            :hover { text-decoration: underline; }
        "},
        (ButtonVariant::Text, ButtonColor::Secondary) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 600;
            color: #6c757d;
            :hover { text-decoration: underline; }
        "},
        (ButtonVariant::Outline, ButtonColor::Danger) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 700;
            background: #fff; padding: 4px 12px; border-radius: 4px;
            border: 1px solid #dc3545; color: #dc3545;
            :hover { background: #dc3545; color: #fff; }
        "},
        (ButtonVariant::Outline, ButtonColor::Primary) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 700;
            background: #fff; padding: 4px 12px; border-radius: 4px;
            border: 1px solid #007bff; color: #007bff;
            :hover { background: #007bff; color: #fff; }
        "},
        (ButtonVariant::Outline, ButtonColor::Success) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 700;
            background: #fff; padding: 4px 12px; border-radius: 4px;
            border: 1px solid #28a745; color: #28a745;
            :hover { background: #28a745; color: #fff; }
        "},
        (ButtonVariant::Outline, ButtonColor::Secondary) => css! {"
            font-size: 0.8rem; cursor: pointer; font-weight: 700;
            background: #fff; padding: 4px 12px; border-radius: 4px;
            border: 1px solid #6c757d; color: #6c757d;
            :hover { background: #6c757d; color: #fff; }
        "},
    };

    dom! {
        <div
            on_click={move |_| on_click()}
            css={style}
        >
            {label}
        </div>
    }
}

#[component]
pub fn TableButton(label: String, on_click: Rc<dyn Fn() + 'static>) {
    dom! {
        <div
            on_click={move |_| on_click()}
            css={css! {"
                padding: 8px 16px;
                background: #232323;
                color: #fff;
                border-radius: 8px;
                font-size: 0.9rem;
                font-weight: 600;
                cursor: pointer;
                transition: all 0.2s;
                :hover { background: #444; }
            "}}
        >
            { label }
        </div>
    }
}
