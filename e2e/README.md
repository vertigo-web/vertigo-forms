# vertigo-forms e2e

Browser tests of the storybook (`storybook/`), story by story. They drive Chrome through
[thirtyfour](https://docs.rs/thirtyfour) against the storybook built from the current
workspace, so they check the whole path: server-side rendering, the start of the WASM app and
how the components behave.

## Requirements

- Google Chrome. On the first run thirtyfour downloads the matching chromedriver to
  `~/.cache/thirtyfour`.
- The same toolchain as the rest of the workspace: nightly with the `wasm32-unknown-unknown`
  target.

`vertigo-cli` doesn't have to be installed: the suite builds and serves the storybook with the
vertigo-cli library.

## Running

```sh
cargo test -p vertigo-forms-e2e                       # all tests
cargo test -p vertigo-forms-e2e -- resource_table     # only tests with "resource_table" in the name
cargo test -p vertigo-forms-e2e -- --include-ignored  # with the tests of known bugs
```

The `Justfile` in the workspace root has it shorter: `just e2e`, which takes the same arguments
(`just e2e resource_table`), and `just e2e-headed`, which shows the browser.

Plain `cargo test` in the workspace root runs only the library's tests, and CI excludes this
crate from its test run. Use `cargo test`, not nextest: nextest runs each test in a separate
process, and the suite builds and serves the storybook once for all of them.

The suite:

1. Builds the storybook into `target/e2e/<profile>/build`, as `vertigo build` does, but without
   `wasm-opt`.
2. Serves it in the test process, as `vertigo serve` does (SSR included), on a free port.
3. Gives each test its own Chrome session. By default up to 4 tests run at once, which
   `--test-threads` changes.

When a test fails, a screenshot, the page source, the console log and the error land in
`target/e2e/storybook/artifacts/<test>/`. A test also fails when the browser console reports an
error (a panic in WASM, for example), unless the test allows it with `ctx.allow_console(...)`.

The browser can't reach the internet. The only outside resources are the sample pictures from
picsum.photos, which don't load, and the tests that show them allow that error with
`ctx.allow_sample_pictures()`.

## Environment variables

| Variable | Effect |
|---|---|
| `E2E_HEADLESS=0` | Shows the browser window |
| `E2E_SKIP_BUILD=1` | Uses the last build |
| `E2E_RELEASE=1` | Builds the storybook in release mode |
| `E2E_CHROMEDRIVER` | Path to chromedriver to use instead of downloading it |

## Writing tests

- Tests of a story are in `tests/storybook/<story>.rs`, named like the story's module in
  `storybook/src/`, and `shell.rs` tests the storybook itself. A test is
  `async fn name(ctx: Ctx) -> Result<()>`, registered in `tests/storybook/main.rs` with
  `browser_tests!`.
- `ctx.open(path)` waits until the app takes over the page rendered by the server. Before that,
  nothing on the page reacts to clicks or typing.
- Each test gets a fresh page and the stories keep their state in the page, so tests don't
  affect each other.
- Find elements by their text, structure or attributes like `name` and `title`. Not by
  `autocss_N` classes, which depend on the order of rendering, nor by `v-component` and `v-css`,
  which only debug builds have.
- vertigo renders parts of the page anew when their values change, so an element found
  earlier can be stale. Find it again, or use the `wait_*` helpers, which do.
- Type with `ctx.retype`. WebDriver's `clear()` doesn't send an `input` event, so vertigo would
  keep the old value.
- `ResourceTable`: `ctx.table(title)` gives `rows`, `wait_for_rows`, `click_in_row`,
  `open_add_form`, `form`, `forms`, `click_form_button` and `answer_delete`. A row in view mode
  is found by `data-testid="row-buttons"` from vertigo-forms, an open form by its "Cancel"
  button and its fields.
- `ctx.mark_document()` and later `ctx.assert_same_document()` check that a navigation happened
  in the app, without loading a new page.
- A test of a known, not yet fixed bug describes the correct behavior and goes to
  `known_failures(...)` in `main.rs`. It's skipped by default and runs with
  `--include-ignored`. Once the bug is fixed, move the test to the regular list.
