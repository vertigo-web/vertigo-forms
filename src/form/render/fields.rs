use vertigo::{AttrGroup, Css, DomNode, Value, css, dom};

use crate::ValidationErrors;

use super::super::{DataField, DataSection, Field, FieldsetStyle};

pub(in super::super) fn fields<'a>(
    sections: &'a [DataSection],
    s: &'a AttrGroup,
    validation_errors: Value<ValidationErrors>,
    section_css: &'a Css,
) -> impl Iterator<Item = DomNode> + 'a {
    let fieldset_flex_css = css! {"
        display: flex;
        gap: 5px;
    "};

    sections.iter().flat_map(move |section| {
        let attrs = s.clone();
        let custom_fieldset_css = section.fieldset_css.clone().unwrap_or_else(|| css! {""});

        let section_rendered = if section.fields.len() > 1 {
            let mut values = vec![];
            for (i, field) in section.fields.iter().enumerate() {
                if section.fieldset_style == FieldsetStyle::Dimensions && i > 0 {
                    values.push(dom! { <span>"x"</span> });
                }
                values.push(render_field(field, &validation_errors));
            }

            dom! {
                <label css={section_css} {..attrs}>
                    {&section.label}
                    <div css={&fieldset_flex_css} css={custom_fieldset_css}>
                        {..values}
                    </div>
                </label>
            }
        } else if let Some(field) = section.fields.first() {
            dom! {
                <label css={section_css} {..attrs}>
                    {&section.label}
                    {render_field(field, &validation_errors)}
                </label>
            }
        } else {
            dom! { <p /> }
        };

        if section.new_group {
            vec![
                dom! { <hr css={css!{"width: 100%; grid-column: 1 / 3;"}}/> },
                section_rendered,
            ]
        } else {
            vec![section_rendered]
        }
    })
}

fn render_field(field: &DataField, validation_errors: &Value<ValidationErrors>) -> DomNode {
    let val_error = {
        let field_key = field.key.to_owned();
        validation_errors.render_value_option(move |errs| {
            errs.get(&field_key).map(|err| dom! { <span>{err}</span> })
        })
    };
    dom! {
        <div css={css!{"display: flex; flex-flow: column nowrap;"}}>
            <Field {field} />
            <span css={css!{"color: red;"}}>{val_error}</span>
        </div>
    }
}
