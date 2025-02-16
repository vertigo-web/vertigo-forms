use std::{collections::HashMap, fmt::Display, str::FromStr};

mod drop_image_file;
pub mod form;
mod input;
pub mod login;
mod popup;
mod search_panel;
mod select;
mod select_search;
mod spinner;
mod switch;
mod tabs;
mod with_loader;

pub use {
    drop_image_file::{image_as_uri, name_to_mime, DropImageFile, DropImageFileParams},
    input::{Input, InputWithButton, InputWithButtonParams, ListInput},
    popup::{Popup, PopupOnHover, PopupParams},
    search_panel::{SearchPanel, SearchPanelParams, SearchResult},
    select::{DictSelect, MultiDropDown, MultiDropDownParams, MultiSelect, Select},
    select_search::{SelectSearch, SelectSearchParams},
    spinner::Spinner,
    switch::{Switch, SwitchParams},
    tabs::{Tab, Tabs, TabsContent, TabsContentMapped, TabsHeader, TabsParams},
    with_loader::{with_loader, WithLoader},
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
