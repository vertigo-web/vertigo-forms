use std::{collections::HashMap, fmt::Display, str::FromStr};

mod drop_image_file;
mod input;
mod search_panel;
mod select;
mod select_search;
mod switch;
mod tabs;

pub use {
    drop_image_file::{image_as_uri, name_to_mime, DropImageFile, DropImageFileParams},
    input::{Input, InputWithButton, InputWithButtonParams},
    search_panel::{SearchPanel, SearchPanelParams, SearchResult},
    select::Select,
    select_search::{SelectSearch, SelectSearchParams},
    switch::{Switch, SwitchParams},
    tabs::{Tab, Tabs, TabsContent, TabsHeader, TabsHeaderParams},
};

pub type ValidationErrors = HashMap<String, String>;

pub fn parse<T>(value: String, field: &'static str, errors: &mut ValidationErrors) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    match value.parse() {
        Ok(price) => Some(price),
        Err(err) => {
            errors.insert(field.to_string(), err.to_string());
            None
        }
    }
}

pub fn nonify(value: String) -> Option<String> {
    Some(value).filter(|v| !v.trim().is_empty())
}

pub fn parse_optional<T>(
    value: String,
    field: &'static str,
    errors: &mut ValidationErrors,
) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    match nonify(value) {
        Some(value) => parse(value, field, errors),
        None => None,
    }
}
