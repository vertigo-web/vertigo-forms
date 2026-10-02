//! Story "Search Panel": `SearchPanel` counting the words of the query, from no letters up.

use vertigo_forms_e2e::prelude::*;

pub async fn shows_result_for_query(ctx: Ctx) -> Result<()> {
    let panel = "//div[text()[normalize-space(.)='Enter words:']]";
    let result = By::XPath(format!("{panel}/div"));

    ctx.open("/search_panel").await?;
    ctx.wait_for_text_of(result.clone(), "No results").await?;

    let input = ctx.find(By::XPath(format!("{panel}/input"))).await?;
    input.send_keys("one").await?;
    ctx.wait_for_text_of(result.clone(), "Word count: 1")
        .await?;
    input.send_keys(" two  three").await?;
    ctx.wait_for_text_of(result.clone(), "Word count: 3")
        .await?;
    ctx.retype(&input, "").await?;
    ctx.wait_for_text_of(result, "No results").await
}
