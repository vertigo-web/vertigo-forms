//! The vertigo-forms storybook, story by story. Every test gets a fresh page, and the stories
//! keep their state in the page only, so the tests can run in parallel.

mod drop_image_file;
mod form;
mod input;
mod login;
mod multi_drop_down;
mod multi_select;
mod popup;
mod resource_table;
mod search_panel;
mod select;
mod select_search;
mod shell;
mod spinner;
mod switch;
mod tabs;
mod with_loader;
mod with_stable_loader;

use vertigo_forms_e2e::{browser_tests, prelude::*};

fn main() {
    let tests = browser_tests![
        shell::every_story_opens_from_its_path,
        shell::server_renders_the_story_of_the_path,
        shell::tabs_switch_stories_in_app,
        shell::back_and_forward_follow_history,
        shell::unknown_path_shows_first_story,
        shell::stories_hydrate_without_mismatches,
        input::input_sets_value_as_typed,
        input::input_with_button_sets_value_on_ok,
        input::list_input_splits_on_commas,
        popup::switch_shows_and_hides_popup,
        popup::hover_shows_popup,
        switch::switches_share_the_value,
        select::choosing_option_sets_value,
        multi_select::buttons_toggle_values,
        multi_drop_down::drop_down_toggles_values,
        select_search::typing_filters_and_click_selects,
        select_search::arrows_and_enter_select,
        select_search::leaving_field_closes_options,
        search_panel::shows_result_for_query,
        form::form_1_submits_changes,
        form::form_1_submits_on_enter,
        form::form_2_submits_selects,
        form::form_2_submits_chosen_photo,
        form::form_2_reverts_photo_without_submitting,
        form::tabbed_form_keeps_values_across_tabs,
        form::tabbed_form_submits_fields_of_all_tabs,
        resource_table::tables_show_their_items,
        resource_table::edit_saves_item,
        resource_table::cancel_drops_changes,
        resource_table::empty_name_is_rejected,
        resource_table::add_appends_items,
        resource_table::add_rejects_empty_name_and_cancel_closes,
        resource_table::delete_asks_first,
        resource_table::rows_keep_their_own_forms,
        resource_table::data_section_form_edits_all_fields,
        resource_table::data_section_cancel_drops_changes,
        resource_table::data_section_add_appends_items,
        resource_table::data_section_rejects_empty_name,
        resource_table::data_section_delete,
        resource_table::rejected_save_returns_to_form,
        resource_table::cancel_after_rejected_save,
        resource_table::rejected_delete_brings_row_back,
        resource_table::rejected_add_returns_to_form,
        resource_table::data_section_rejected_save_keeps_fields,
        tabs::header_switches_tabs,
        tabs::sub_views_show_their_tab,
        drop_image_file::shows_original_picture,
        drop_image_file::chosen_file_replaces_picture_until_reverted,
        drop_image_file::dropped_file_replaces_picture,
        login::wrong_password_shows_error,
        login::right_password_logs_in,
        login::enter_logs_in,
        login::custom_form_logs_in,
        spinner::spinner_is_animated,
        with_loader::follows_resource,
        with_stable_loader::both_follow_resource_until_ready,
        with_stable_loader::stable_loader_keeps_typed_text,
    ];
    // A test of a known, not yet fixed bug goes to
    // `tests.extend(known_failures(browser_tests![...]))` (see README).

    run_suite("storybook", tests);
}
