use vertigo::{bind_rc, main, prelude::*};
use vertigo_forms::form::{DataSection as DS, FormData, FormExport, ModelForm};

#[derive(Clone, Default, PartialEq)]
struct Car {
    brand: String,
    model: String,
    year: String,
    engine: String,
}

impl From<Car> for FormData {
    fn from(value: Car) -> Self {
        FormData::default()
            .add_top_controls()
            .with(DS::with_string_field("Brand", "brand", &value.brand))
            .with(DS::with_string_field("Model", "model", &value.model))
            .with(DS::with_string_field("Year", "year", &value.year))
            .with(DS::new("Engine").add_list_field(
                "engine",
                Some(&value.engine),
                vec!["petrol".into(), "diesel".into(), "electric".into()],
            ))
    }
}

impl From<FormExport> for Car {
    fn from(value: FormExport) -> Self {
        vertigo::log::info!("FormExport {}", value.get_string("engine"));
        Self {
            brand: value.get_string("brand"),
            model: value.get_string("model"),
            year: value.get_string("year"),
            engine: value.list_or_default("engine"),
        }
    }
}

#[main]
fn render() -> DomNode {
    let car = Value::new(Car::default());

    let data_formatted =
        car.map(|car| format!("{} {}, {} ({})", car.brand, car.model, car.engine, car.year));

    let on_submit = bind_rc!(car, |new_model: Car| {
        car.set(new_model);
    });

    dom! {
        <html>
            <head />
            <body>
                <ModelForm model={car} {on_submit} params={} />
                <p>"Form data: " {data_formatted}</p>
            </body>
        </html>
    }
}
