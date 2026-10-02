//! Story "Login": two `Login` forms sharing one result, and a third, built from the custom
//! render slots of `LoginParams`. Only "test" with "123" logs in.

use vertigo_forms_e2e::prelude::*;

const ERROR: &str = "Invalid password, try test/123";
const SUCCESS: &str = "Login successful, token qwerty1234";

/// A default form, found by the label of its user name field.
fn form(username_label: &str) -> String {
    format!(
        "//div[div/div[text()[normalize-space(.)={}]]]",
        xpath_literal(username_label)
    )
}

const CUSTOM: &str = "//div[h3[normalize-space(.)='Custom login form']]";

async fn fill(ctx: &Ctx, form: &str, username: &str, password: &str) -> Result<()> {
    let field = ctx
        .find(By::XPath(format!("{form}//input[not(@type)]")))
        .await?;
    ctx.retype(&field, username).await?;
    ctx.retype(&password_field(ctx, form).await?, password)
        .await
}

async fn password_field(ctx: &Ctx, form: &str) -> Result<WebElement> {
    ctx.find(By::XPath(format!("{form}//input[@type='password']")))
        .await
}

async fn submit(ctx: &Ctx, form: &str) -> Result<()> {
    ctx.find(By::XPath(format!("{form}//input[@type='submit']")))
        .await?
        .click()
        .await?;
    Ok(())
}

/// The line above the fields, where a form shows the error.
fn message(form: &str) -> By {
    By::XPath(format!("{form}/div[1]"))
}

/// What the story shows about the outcome, below the form.
fn outcome(form: &str) -> By {
    By::XPath(format!("{form}/following-sibling::p"))
}

pub async fn wrong_password_shows_error(ctx: Ctx) -> Result<()> {
    let (default, email) = (form("Username:"), form("E-Mail"));

    ctx.open("/login").await?;
    fill(&ctx, &default, "test", "wrong").await?;
    submit(&ctx, &default).await?;
    ctx.wait_for_text_of(outcome(&email), &format!("Login error: {ERROR}"))
        .await?;
    // Both forms show the result they share
    ctx.wait_for_text_of(message(&default), ERROR).await?;
    ctx.wait_for_text_of(message(&email), ERROR).await?;

    ensure!(
        ctx.text_of(message(CUSTOM)).await?.is_empty(),
        "the custom form shows an error too"
    );
    ensure!(
        ctx.find_all(outcome(CUSTOM)).await?.is_empty(),
        "the custom form has an outcome"
    );
    Ok(())
}

pub async fn right_password_logs_in(ctx: Ctx) -> Result<()> {
    let (default, email) = (form("Username:"), form("E-Mail"));

    ctx.open("/login").await?;
    fill(&ctx, &default, "test", "wrong").await?;
    submit(&ctx, &default).await?;
    ctx.wait_for_text_of(message(&default), ERROR).await?;

    ctx.retype(&password_field(&ctx, &default).await?, "123")
        .await?;
    submit(&ctx, &default).await?;
    ctx.wait_for_text_of(outcome(&email), SUCCESS).await?;
    ctx.wait_for_text_of(message(&default), "").await
}

pub async fn enter_logs_in(ctx: Ctx) -> Result<()> {
    let email = form("E-Mail");

    ctx.open("/login").await?;
    fill(&ctx, &email, "test", "123").await?;
    password_field(&ctx, &email)
        .await?
        .send_keys(Key::Enter)
        .await?;
    ctx.wait_for_text_of(outcome(&email), SUCCESS).await
}

pub async fn custom_form_logs_in(ctx: Ctx) -> Result<()> {
    ctx.open("/login").await?;
    let texts = ctx
        .texts(By::XPath(format!(
            "{CUSTOM}/*[self::h3 or self::p]|{CUSTOM}//label"
        )))
        .await?;
    ensure!(
        texts
            == [
                "Custom login form",
                "👤 Username",
                "🔒 Password",
                "Forgot your password?"
            ],
        "texts of the custom form: {texts:?}"
    );

    fill(&ctx, CUSTOM, "test", "wrong").await?;
    ctx.find(By::XPath(format!(
        "{CUSTOM}//button[normalize-space(.)='Sign in →']"
    )))
    .await?
    .click()
    .await?;
    ctx.wait_for_text_of(outcome(CUSTOM), &format!("Login error: {ERROR}"))
        .await?;
    ctx.wait_for_text_of(message(CUSTOM), ERROR).await?;
    ensure!(
        ctx.find_all(outcome(&form("E-Mail"))).await?.is_empty(),
        "the other forms got the outcome"
    );

    let password = password_field(&ctx, CUSTOM).await?;
    ctx.retype(&password, "123").await?;
    password.send_keys(Key::Enter).await?;
    ctx.wait_for_text_of(outcome(CUSTOM), SUCCESS).await?;
    ctx.wait_for_text_of(message(CUSTOM), "").await
}
