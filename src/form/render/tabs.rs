use std::rc::Rc;
use vertigo::{AttrGroup, Css, DomNode, Value, dom};

use crate::{
    Tab, Tabs, TabsParams, ValidationErrors,
    form::{DataSection, fields},
};

pub(in super::super) fn tabs<'a>(
    tabs: &'a [(String, Rc<Vec<DataSection>>)],
    tabs_params: &'a Option<TabsParams>,
    s: &'a AttrGroup,
    validation_errors: Value<ValidationErrors>,
    section_css: &'a Css,
    form_css: &Css,
) -> Option<DomNode> {
    if tabs.is_empty() {
        return None;
    }

    let current_tab = Value::new(tabs[0].0.clone());

    let tabs = tabs
        .iter()
        .map(|(label, sections)| Tab {
            key: label.clone(),
            name: label.to_string(),
            render: {
                let sections = sections.clone();
                let s = s.clone();
                let validation_errors = validation_errors.clone();
                let section_css = section_css.clone();
                let form_css = form_css.clone();
                Rc::new(move |_| {
                    let fields = fields(&sections, &s, validation_errors.clone(), &section_css);
                    dom! {
                        <div css={&form_css}>
                            {..fields}
                        </div>
                    }
                })
            },
        })
        .collect::<Vec<_>>();

    type FormTabs = Tabs<Value<String>, String>;

    Some(dom! {
        <FormTabs
            tabs={tabs}
            current_tab={current_tab}
            params={tabs_params.clone().unwrap_or_default()}
        />
    })
}
