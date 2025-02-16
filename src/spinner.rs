use vertigo::{component, css, dom};

#[component]
pub fn Spinner() {
    dom! { <div css={css! {"
        width: 40px;
        height: 40px;
        margin: 100px auto;
        background-color: #d26913;

        border-radius: 100%;
        animation: 1.0s infinite ease-in-out {
            0% {
                -webkit-transform: scale(0);
                transform: scale(0);
            } 100% {
                -webkit-transform: scale(1.0);
                transform: scale(1.0);
                opacity: 0;
            }
        };
    "}} /> }
}
