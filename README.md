# Vertigo Forms

Blocks for building forms in [vertigo](https://crates.io/crates/vertigo).

[![crates.io](https://img.shields.io/crates/v/vertigo-forms)](https://crates.io/crates/vertigo-forms)
[![Documentation](https://docs.rs/vertigo-forms/badge.svg)](https://docs.rs/vertigo-forms)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/vertigo-forms.svg)
[![Dependency Status](https://deps.rs/crate/vertigo-forms/0.1.0/status.svg)](https://deps.rs/crate/vertigo-forms/0.1.0)
[![CI](https://github.com/vertigo-web/vertigo-forms/actions/workflows/pipeline.yaml/badge.svg)](https://github.com/vertigo-web/vertigo-forms/actions/workflows/pipeline.yaml)
[![downloads](https://img.shields.io/crates/d/vertigo-forms.svg)](https://crates.io/crates/vertigo-forms)

See [Changelog](https://github.com/vertigo-web/vertigo-forms/blob/master/CHANGES.md) for recent features.

## Example

Dependencies:

```toml
vertigo = "0.8"
vertigo-forms = { git = "https://github.com/vertigo-web/vertigo-forms" }
```

Example:

```rust
use vertigo::{computed_tuple, main, prelude::*};
use vertigo_forms::{Input, Select};

#[derive(Default)]
struct FormData {
    brand: Value<String>,
    model: Value<String>,
    year: Value<String>,
    engine: Value<String>,
}

#[main]
fn render() -> DomNode {
    let form_data = FormData::default();

    let engine_types = Value::new(vec![
        "petrol".to_string(),
        "diesel".to_string(),
        "electric".to_string(),
    ]);

    let data_formatted = computed_tuple!(
        brand => form_data.brand,
        model => form_data.model,
        engine => form_data.engine,
        year => form_data.year
    )
        .map(|(brand, model, engine, year)|
            format!("{} {}, {} ({})", brand, model, engine, year)
        );

    dom! {
        <html>
            <head />
            <body>
                <form>
                    <p>"Brand: " <Input value={form_data.brand} /></p>
                    <p>"Model: " <Input value={form_data.model} /></p>
                    <p>"Year: " <Input value={form_data.year} /></p>
                    <p>"engine: " <Select value={form_data.engine} options={engine_types} /></p>
                </form>
                <p>
                    "Form data: " {data_formatted}
                </p>
            </body>
        </html>
    }
}
```

## Storybook App

### Prepare

Make sure you're using nightly version of rust:

* `rustup default nightly`

Install vertigo-cli:

* `cargo install vertigo-cli`

### Run

Build and run storybook in watch mode:

* `vertigo watch vertigo-forms-storybook`

Eventually terminal will let you know that app is available under `http://localhost:4444/`

If you want to play around with the code, the browser will automatically refresh after the project has been recompiled.

### Run example

To run the example in watch mode (it will run also on localhost:4444):
`vertigo watch vertigo-forms-example-form`
