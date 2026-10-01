use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};
use vertigo::{Computed, DomComment, DomNode, Value, bind_rc, bind_spawn, css, dom, transaction};

use crate::button::{Button, ButtonColor, ButtonVariant};

pub type AsyncResult<R> = Pin<Box<dyn Future<Output = R>>>;
pub type ProcessCallback<Model> = Rc<dyn Fn(Model) -> AsyncResult<Option<String>>>;
pub type CancelCallback = Rc<dyn Fn() -> AsyncResult<Option<String>>>;
pub type CreateFn = Rc<dyn Fn() -> DomNode>;
pub type RowResult<T> = Result<T, Vec<String>>;

#[derive(Clone, PartialEq)]
pub enum RowState {
    View { confirm_delete: bool },
    Edit,
    Processing,
}

#[derive(Clone)]
pub struct ResourceTableLabels {
    pub edit: String,
    pub delete: String,
    pub confirm_delete: String,
    pub cancel: String,
    pub confirm_question: String,
    pub processing: String,
    pub save: String,
}

impl Default for ResourceTableLabels {
    fn default() -> Self {
        Self {
            edit: "Edit".to_string(),
            delete: "Delete".to_string(),
            confirm_delete: "Confirm delete".to_string(),
            cancel: "Cancel".to_string(),
            confirm_question: "Are you sure you want to delete this record?".to_string(),
            processing: "Processing...".to_string(),
            save: "Save".to_string(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn row_form<Model: Clone + PartialEq + 'static, FormModel: Clone + 'static>(
    initial_state: RowState,
    item: &Model,
    create_form_model: fn(&Model) -> FormModel,
    update_model: fn(&Model, &FormModel, &vertigo::Context) -> RowResult<Model>,
    render_view: fn(&Model, create_buttons: CreateFn, alert: Computed<bool>) -> DomNode,
    render_form: fn(&FormModel, buttons: DomNode) -> DomNode,

    process_label: String,
    process: ProcessCallback<Model>,

    cancel_label: String,
    cancel: CancelCallback,

    delete: Option<ProcessCallback<Model>>,

    labels: ResourceTableLabels,
) -> DomNode {
    row_form_with_store(
        RowStore::new(initial_state),
        item,
        create_form_model,
        update_model,
        render_view,
        render_form,
        process_label,
        process,
        cancel_label,
        cancel,
        delete,
        labels,
    )
}

/// State of a row kept outside of its rendering.
///
/// A row is rendered anew whenever its item changes or comes back: an optimistic update or
/// removal, its rollback after a failure, a background refresh. [`ResourceTable`](super::ResourceTable)
/// keeps one store per list position, so the row stays in the edit form with what was typed
/// in, and the message about a failed save or delete is shown in the row that comes back.
#[derive(Clone)]
pub(crate) struct RowStore<Model: Clone + PartialEq + 'static, FormModel: Clone + 'static> {
    state: Value<RowState>,
    error: Value<Option<String>>,
    /// Created from the item when editing starts, so that the next edit doesn't start from
    /// changes dropped with "Cancel". A failed save returns to the same form.
    form_model: Rc<RefCell<Option<FormModel>>>,
    optimistic_item: Value<Option<Model>>,
}

impl<Model: Clone + PartialEq + 'static, FormModel: Clone + 'static> RowStore<Model, FormModel> {
    pub(crate) fn new(initial_state: RowState) -> Self {
        Self {
            state: Value::new(initial_state),
            error: Value::new(None),
            form_model: Rc::new(RefCell::new(None)),
            optimistic_item: Value::new(None),
        }
    }
}

/// [`row_form`] with its state in `store`, owned by the caller.
#[allow(clippy::too_many_arguments)]
pub(crate) fn row_form_with_store<
    Model: Clone + PartialEq + 'static,
    FormModel: Clone + 'static,
>(
    store: RowStore<Model, FormModel>,
    item: &Model,
    create_form_model: fn(&Model) -> FormModel,
    update_model: fn(&Model, &FormModel, &vertigo::Context) -> RowResult<Model>,
    render_view: fn(&Model, create_buttons: CreateFn, alert: Computed<bool>) -> DomNode,
    render_form: fn(&FormModel, buttons: DomNode) -> DomNode,

    process_label: String,
    process: ProcessCallback<Model>,

    cancel_label: String,
    cancel: CancelCallback,

    delete: Option<ProcessCallback<Model>>,

    labels: ResourceTableLabels,
) -> DomNode {
    let RowStore {
        state,
        error,
        form_model,
        optimistic_item,
    } = store;

    let alert_view = state.map(|value| {
        if let RowState::View { confirm_delete } = value
            && !confirm_delete
        {
            false
        } else {
            true
        }
    });

    let item = item.clone();
    state.clone().render_value(move |current_state| {
        let error_view = error.clone().render_value(|err| {
            if let Some(err) = err {
                dom! { <div css={css! {"color: #dc3545; font-size: 0.8rem; margin: 8px 20px; font-weight: 600;"}}>{err}</div> }
            } else {
                dom! { <div /> }
            }
        });

        match current_state {
            RowState::View { confirm_delete } => {
                let confirm_view = if confirm_delete {
                    let confirm = bind_rc!(state, error, delete, item, || {
                        if let Some(delete_fn) = &delete {
                            state.set(RowState::Processing);

                            bind_spawn!(state, error, delete_fn, item, async move {
                                let result = delete_fn(item).await;
                                if let Some(err) = result {
                                    error.set(Some(err));
                                    state.set(RowState::View { confirm_delete: false });
                                } else {
                                    error.set(None);
                                    state.set(RowState::View { confirm_delete: false });
                                }
                            })();
                        }
                    });

                    confirm_delete_view(state.clone(), confirm, &labels)
                } else {
                    dom! {
                        <div />
                    }
                };

                let state_for_buttons = state.clone();
                let error_for_buttons = error.clone();
                let form_model_for_buttons = form_model.clone();
                let item_for_buttons = item.clone();
                let delete_for_buttons = delete.clone();
                let edit_label = labels.edit.clone();
                let delete_label = labels.delete.clone();

                let create_buttons = move || {
                    let delete_view_inner = if delete_for_buttons.is_some() {
                        let delete_click = bind_rc!(state_for_buttons, error_for_buttons, || {
                            error_for_buttons.set(None);
                            state_for_buttons.set(RowState::View { confirm_delete: true });
                        });

                        dom! {
                            <Button
                                label={delete_label.clone()}
                                on_click={delete_click}
                                color={ButtonColor::Danger}
                                variant={ButtonVariant::Outline}
                            />
                        }
                    } else {
                        dom! { <div /> }
                    };

                    let edit_click = bind_rc!(
                        state_for_buttons,
                        error_for_buttons,
                        form_model_for_buttons,
                        item_for_buttons,
                        || {
                            *form_model_for_buttons.borrow_mut() =
                                Some(create_form_model(&item_for_buttons));
                            error_for_buttons.set(None);
                            state_for_buttons.set(RowState::Edit);
                        }
                    );

                    dom! {
                        <div data-testid="row-buttons" css={css! {"display: flex; gap: 12px;"}}>
                            <Button
                                label={edit_label.clone()}
                                on_click={edit_click}
                                color={ButtonColor::Primary}
                                variant={ButtonVariant::Outline}
                            />
                            {delete_view_inner}
                        </div>
                    }
                };

                DomComment::dom_fragment(vec![
                    render_view(&item, Rc::new(create_buttons), alert_view.clone()),
                    dom! {
                        <div>
                            { confirm_view }
                            { error_view }
                        </div>
                    },
                ])
                .into()
            }
            RowState::Edit => {
                let form = form_model
                    .borrow_mut()
                    .get_or_insert_with(|| create_form_model(&item))
                    .clone();

                let buttons = dom! {
                    <div css={css! {"display: flex; gap: 8px; justify-content: flex-end;"}}>
                        <Button
                            label={process_label.clone()}
                            on_click={bind_rc!(state, error, item, process, form, optimistic_item, || {
                                let result = transaction(|ctx| {
                                    update_model(&item, &form, ctx)
                                });

                                match result {
                                    Ok(new_model) => {
                                        optimistic_item.set(Some(new_model.clone()));
                                        state.set(RowState::Processing);
                                        bind_spawn!(state, error, optimistic_item, process, async move {
                                            let result = process(new_model).await;
                                            if let Some(err) = result {
                                                error.set(Some(err));
                                                optimistic_item.set(None);
                                                state.set(RowState::Edit);
                                            } else {
                                                error.set(None);
                                                state.set(RowState::View { confirm_delete: false });
                                                optimistic_item.set(None);
                                            }
                                        })();
                                    }
                                    Err(errors) => {
                                        error.set(Some(errors.join(", ")));
                                    }
                                }
                            })}
                            color={ButtonColor::Success}
                            variant={ButtonVariant::Outline}
                        />
                        <Button
                            label={cancel_label.clone()}
                            on_click={bind_rc!(state, error, cancel, || {
                                bind_spawn!(cancel, async move {
                                    cancel().await;
                                })();
                                // The error was about the edit being dropped
                                error.set(None);
                                state.set(RowState::View { confirm_delete: false });
                            })}
                            color={ButtonColor::Danger}
                            variant={ButtonVariant::Outline}
                        />
                    </div>
                };

                let rendered_form = render_form(&form, buttons);

                dom! {
                    <div>
                        { rendered_form }
                        { error_view }
                    </div>
                }
            }
            RowState::Processing => {
                let processing_label = labels.processing.clone();
                optimistic_item.clone().render_value(move |opt_item| {
                    if let Some(opt_item) = opt_item {
                        let no_alert = Computed::from(|_ctx| false);
                        render_view(&opt_item, Rc::new(|| dom! { <div /> }), no_alert)
                    } else {
                        let label = processing_label.clone();
                        dom! {
                            <div css={css! {"
                                padding: 24px 20px;
                                text-align: center;
                                color: #888;
                                background: #f9f9f9;
                                border-bottom: 1px solid #eee;
                            "}}>
                                <span css={css! {"
                                    display: inline-block;
                                    animation: spin 1s linear infinite;
                                    margin-right: 8px;
                                "}}>"↻"</span>
                                {label}
                            </div>
                        }
                    }
                })
            }
        }
    })
}

fn confirm_delete_view(
    state: Value<RowState>,
    on_confirm: Rc<dyn Fn() + 'static>,
    labels: &ResourceTableLabels,
) -> DomNode {
    let confirm_question = labels.confirm_question.clone();
    let confirm_delete_label = labels.confirm_delete.clone();
    let cancel_label = labels.cancel.clone();

    dom! {
        <div css={css! {"
            background: #fff0f0;
            padding: 16px 20px;
            border-bottom: 1px solid #ffcccc;

            display: flex;
            flex-direction: row;
            justify-content: space-between;
        "}}>
            <div css={css! {"
                font-weight: 600;
                color: #b02a37;

                display: flex;
                align-items: center;
            "}}>
                {confirm_question}
            </div>

            <div css={css! {"
                display: flex;
                gap: 12px;
                justify-content: flex-end;
            "}}>
                <Button
                    label={confirm_delete_label}
                    on_click={on_confirm}
                    color={ButtonColor::Danger}
                    variant={ButtonVariant::Outline}
                />
                <Button
                    label={cancel_label}
                    on_click={bind_rc!(state, || {
                        state.set(RowState::View { confirm_delete: false });
                    })}
                    color={ButtonColor::Secondary}
                    variant={ButtonVariant::Outline}
                />
            </div>
        </div>
    }
}
