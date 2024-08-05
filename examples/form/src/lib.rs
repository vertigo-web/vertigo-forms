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
    .map(|(brand, model, engine, year)| format!("{} {}, {} ({})", brand, model, engine, year));

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
