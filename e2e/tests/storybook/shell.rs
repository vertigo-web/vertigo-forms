//! The storybook itself: `Tabs` over a history router, a story under each path.

use vertigo_forms_e2e::prelude::*;

pub struct Story {
    pub tab: &'static str,
    pub path: &'static str,
    /// XPath of something only this story shows.
    pub marker: &'static str,
}

pub const STORIES: &[Story] = &[
    Story {
        tab: "Input",
        path: "/input",
        marker: "//h4[normalize-space(.)='ListInput']",
    },
    Story {
        tab: "Popup",
        path: "/popup",
        marker: "//p[normalize-space(.)='Popup on hover']",
    },
    Story {
        tab: "Switch",
        path: "/switch",
        marker: "//p[starts-with(normalize-space(.), 'Toggle 2:')]",
    },
    Story {
        tab: "Select",
        path: "/select",
        marker: "//p[normalize-space(.)='Selected value: foo']",
    },
    Story {
        tab: "MultiSelect",
        path: "/multi_select",
        marker: "//button[normalize-space(.)='baz']",
    },
    Story {
        tab: "MultiDropDown",
        path: "/multi_drop_down",
        marker: "//button[normalize-space(.)='V']",
    },
    Story {
        tab: "Select/Search",
        path: "/select_search",
        marker: "//input[@title='Enter phrase']",
    },
    Story {
        tab: "Search Panel",
        path: "/search_panel",
        marker: "//div[text()[normalize-space(.)='Enter words:']]",
    },
    Story {
        tab: "Form",
        path: "/form",
        marker: "//h4[normalize-space(.)='Form 1:']",
    },
    Story {
        tab: "Resource Table",
        path: "/resource_table",
        marker: "//h2[normalize-space(.)='My Resources (DataSection)']",
    },
    Story {
        tab: "Tabs",
        path: "/tabs",
        marker: "//p[normalize-space(.)='View 1 content']",
    },
    Story {
        tab: "Drop Image File",
        path: "/drop_file",
        marker: "//p[normalize-space(.)='Dropped image:']",
    },
    Story {
        tab: "Login",
        path: "/login",
        marker: "//h3[normalize-space(.)='Custom login form']",
    },
    Story {
        tab: "Spinner",
        path: "/spinner",
        // The story is the spinner alone: an empty `div`, right in the tab's content
        marker: "/html/body/div/div/div[not(*) and normalize-space(.)='']",
    },
    Story {
        tab: "With Loader",
        path: "/with_loader",
        marker: "//main[normalize-space(.)='Resource ready: Initial value']",
    },
    Story {
        tab: "With Stable Loader",
        path: "/with_stable_loader",
        marker: "//h3[normalize-space(.)='WithStableLoader']",
    },
];

async fn wait_for_story(ctx: &Ctx, story: &Story) -> Result<()> {
    ctx.find(By::XPath(story.marker.to_string()))
        .await
        .with_context(|| format!("story {:?} isn't shown", story.tab))?;
    ctx.wait_for_tab_active(story.tab, true).await
}

pub async fn every_story_opens_from_its_path(ctx: Ctx) -> Result<()> {
    ctx.allow_sample_pictures();
    for story in STORIES {
        ctx.open(story.path).await?;
        wait_for_story(&ctx, story)
            .await
            .with_context(|| format!("opening {}", story.path))?;
    }
    Ok(())
}

/// The server renders the story of the path itself, before the app starts.
pub async fn server_renders_the_story_of_the_path(ctx: Ctx) -> Result<()> {
    ctx.open("/").await?;
    for story in STORIES {
        // Parsed as the browser parses a page, but without running it
        let found = ctx
            .js_with(
                "const [path, marker] = arguments;
                 return fetch(path)
                     .then(response => response.text())
                     .then(html => {
                         const page = new DOMParser().parseFromString(html, 'text/html');
                         return page.evaluate(marker, page, null, XPathResult.BOOLEAN_TYPE, null)
                             .booleanValue;
                     });",
                vec![json!(story.path), json!(story.marker)],
            )
            .await?;
        ensure!(
            found == Value::Bool(true),
            "the page from the server for {} doesn't show story {:?}",
            story.path,
            story.tab
        );
    }
    Ok(())
}

pub async fn tabs_switch_stories_in_app(ctx: Ctx) -> Result<()> {
    ctx.allow_sample_pictures();
    ctx.open("/").await?;
    wait_for_story(&ctx, &STORIES[0]).await?;
    ctx.mark_document().await?;

    // Backwards, so that each click changes the story, the first one included
    let mut previous = &STORIES[0];
    for story in STORIES.iter().rev() {
        ctx.click_tab(story.tab).await?;
        ctx.wait_for_path(story.path).await?;
        wait_for_story(&ctx, story).await?;
        ctx.wait_for_tab_active(previous.tab, false).await?;
        previous = story;
    }
    ctx.assert_same_document().await
}

pub async fn back_and_forward_follow_history(ctx: Ctx) -> Result<()> {
    let [input, select, tabs] = ["Input", "Select", "Tabs"].map(|tab| {
        STORIES
            .iter()
            .find(|story| story.tab == tab)
            .expect("story listed")
    });

    ctx.open(input.path).await?;
    ctx.mark_document().await?;
    ctx.click_tab(select.tab).await?;
    wait_for_story(&ctx, select).await?;
    ctx.click_tab(tabs.tab).await?;
    wait_for_story(&ctx, tabs).await?;

    ctx.back().await?;
    ctx.wait_for_path(select.path).await?;
    wait_for_story(&ctx, select).await?;
    ctx.back().await?;
    ctx.wait_for_path(input.path).await?;
    wait_for_story(&ctx, input).await?;
    ctx.forward().await?;
    ctx.wait_for_path(select.path).await?;
    wait_for_story(&ctx, select).await?;
    ctx.assert_same_document().await
}

pub async fn unknown_path_shows_first_story(ctx: Ctx) -> Result<()> {
    ctx.open("/no-such-story").await?;
    wait_for_story(&ctx, &STORIES[0]).await
}

/// The app takes over the page rendered by the server without rendering any of it anew.
///
/// A block (`div`, `ul`) in a `<p>` breaks that: the browser's HTML parser closes the `<p>`
/// before the block, so the page differs from what the app renders.
pub async fn stories_hydrate_without_mismatches(ctx: Ctx) -> Result<()> {
    ctx.allow_sample_pictures();
    let mut mismatched = Vec::new();
    for story in STORIES {
        ctx.open(story.path).await?;
        let report = ctx.js("return window.__vertigo_hydration").await?;
        if report["matched"] != report["hydratable"] {
            mismatched.push(format!(
                "{} ({} of {} nodes matched)",
                story.path, report["matched"], report["hydratable"]
            ));
        }
    }
    ensure!(
        mismatched.is_empty(),
        "hydration rendered nodes anew in: {}",
        mismatched.join(", ")
    );
    Ok(())
}
