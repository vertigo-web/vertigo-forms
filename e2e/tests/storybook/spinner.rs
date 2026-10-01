//! Story "Spinner".

use vertigo_forms_e2e::prelude::*;

pub async fn spinner_is_animated(ctx: Ctx) -> Result<()> {
    ctx.open("/spinner").await?;
    let spinner = ctx
        .find(By::XPath("/html/body/div/div/div[not(*)]"))
        .await?;

    for (property, expected) in [
        ("width", "40px"),
        ("height", "40px"),
        ("animation-iteration-count", "infinite"),
    ] {
        let value = ctx.style(&spinner, property).await?;
        ensure!(value == expected, "{property}: {value}");
    }
    let running = ctx
        .js_with(
            "return arguments[0].getAnimations().some(animation => animation.playState === 'running')",
            vec![spinner.to_json()?],
        )
        .await?;
    ensure!(running == Value::Bool(true), "the spinner doesn't spin");
    Ok(())
}
