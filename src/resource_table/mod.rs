use std::rc::Rc;
use vertigo::{Computed, Css, DomNode, Resource, Value, bind_rc, css, dom};

use crate::button::TableButton;

pub mod row_from_data_section;
pub use row_from_data_section::row_from_data_section;
pub mod row_form;

pub use row_form::{
    AsyncResult, CancelCallback, CreateFn, ProcessCallback, ResourceTableLabels, RowResult,
    RowState, row_form,
};

type ComputedListItem<T> = Computed<Option<T>>;
type ComputedList<T> = Computed<Resource<Rc<Vec<ComputedListItem<T>>>>>;

/// A generic CRUD table component with inline add/edit forms.
///
/// `Model` is the read-only row type; `ModelForm` is the editable form state created from it via
/// `create_form_model`.
///
/// ## Feeding `list` from `LazyListCache`
///
/// [`vertigo::LazyListCache::granular`] returns `Resource<Rc<Vec<Computed<Option<T::Value>>>>>`,
/// which matches `list`'s inner type. Wrap it in [`Computed::from`] to get the right shape:
///
/// ```rust,ignore
/// let list = {
///     let cache = my_cache.clone();
///     Computed::from(move |ctx| cache.granular(ctx, None as Option<fn(&MyModel) -> bool>))
/// };
///
/// ResourceTable {
///     list,
///     // ...
/// }
/// ```
#[derive(Clone)]
pub struct ResourceTable<Model: Clone + PartialEq + Default + 'static, ModelForm: Clone + 'static> {
    pub list: ComputedList<Model>,
    pub title: String,
    pub add_label: String,
    pub table_css: Css,
    pub render_header: fn() -> DomNode,
    pub render_filters: Option<fn() -> DomNode>,
    pub create_new_model: Rc<dyn Fn() -> Model>,
    pub create_form_model: fn(&Model) -> ModelForm,
    pub update_model: fn(&Model, &ModelForm, &vertigo::Context) -> RowResult<Model>,
    pub render_row_view: fn(&Model, create_buttons: CreateFn, alert: Computed<bool>) -> DomNode,
    pub render_row_form: fn(&ModelForm, buttons: DomNode) -> DomNode,
    pub on_create: ProcessCallback<Model>,
    pub on_update: ProcessCallback<Model>,
    pub on_delete: Option<ProcessCallback<Model>>,
    pub labels: ResourceTableLabels,
}

impl<Model: Clone + PartialEq + Default + 'static, ModelForm: Clone + 'static>
    ResourceTable<Model, ModelForm>
{
    pub fn into_component(self) -> ResourceTable<Model, ModelForm> {
        self
    }
    pub fn mount(self) -> DomNode {
        let props = Rc::new(self);

        let props2 = props.clone();
        let list = props.list.render_value(move |list| {
            let mut rows = Vec::new();
            let props = props2.clone();
            match list {
                Resource::Ready(list) => {
                    for item in &*list {
                        let props = props.clone();
                        let row = item.render_value_option(move |item| {
                            item.map(|item| {
                                row_form(
                                    RowState::View {
                                        confirm_delete: false,
                                    },
                                    &item,
                                    props.create_form_model,
                                    props.update_model,
                                    props.render_row_view,
                                    props.render_row_form,
                                    props.labels.save.clone(),
                                    props.on_update.clone(),
                                    props.labels.cancel.clone(),
                                    Rc::new(|| Box::pin(async { None })),
                                    props.on_delete.clone(),
                                    props.labels.clone(),
                                )
                            })
                        });

                        rows.push(row);
                    }

                    dom! {
                        <div>
                            { ..rows }
                        </div>
                    }
                }
                Resource::Loading => {
                    dom! { <div>"Loading..."</div> }
                }
                Resource::Error(err) => {
                    dom! { <div>{err}</div> }
                }
            }
        });

        let is_adding = Value::new(None);

        let table_css = css! {"
            width: 100%;
            max-width: 1600px;
            background: #ffffff;
            border: 1px solid #e0e0e0;
            border-radius: 12px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
            overflow: hidden;
            margin: 0 auto;
        "} + &props.table_css;

        dom! {
            <div css={css!{"
                    padding: 10px;
                    width: 100%;
                "}}>
                <div css={table_css}>
                    <div css={css!{"
                            display: flex;
                            justify-content: space-between;
                            align-items: center;
                            padding: 20px;
                            border-bottom: 1px solid #eee;
                            gap: 10px;
                        "}}>
                        <h2 css={css!{"margin: 0; font-size: 1.2rem; color: #333;"}}>{props.title.clone()}</h2>

                        {..props.render_filters.map(|render_filters| render_filters())}

                        <TableButton
                            label={props.add_label.clone()}
                            on_click={bind_rc!(is_adding, props.create_new_model, || {
                                is_adding.set(Some(create_new_model()))
                            })}
                        />

                    </div>

                    { (props.render_header)() }

                    <div>
                        { render_add(props.clone(), is_adding) }
                        { list }
                    </div>
                </div>
            </div>
        }
    }
}

fn render_add<Model: Clone + PartialEq + Default + 'static, ModelForm: Clone + 'static>(
    props: Rc<ResourceTable<Model, ModelForm>>,
    is_adding: Value<Option<Model>>,
) -> DomNode {
    is_adding.clone().render_value({
        let is_adding = is_adding.clone();
        let props = props.clone();

        move |current_is_adding| {
            if let Some(item) = current_is_adding {
                let is_adding = is_adding.clone();
                let props = props.clone();

                row_form(
                    RowState::Edit,
                    &item,
                    props.create_form_model,
                    props.update_model,
                    |_, _, _| dom! { <div /> },
                    props.render_row_form,
                    props.add_label.clone(),
                    Rc::new({
                        let is_adding = is_adding.clone();
                        let on_create = props.on_create.clone();

                        move |item| {
                            let is_adding = is_adding.clone();
                            let on_create = on_create.clone();

                            Box::pin(async move {
                                let result = on_create(item).await;
                                if result.is_none() {
                                    is_adding.set(None);
                                }
                                result
                            })
                        }
                    }),
                    props.labels.cancel.clone(),
                    Rc::new({
                        let is_adding = is_adding.clone();
                        move || {
                            is_adding.set(None);
                            Box::pin(async move { None })
                        }
                    }),
                    None,
                    props.labels.clone(),
                )
            } else {
                dom! { <div /> }
            }
        }
    })
}

pub fn base_header_css() -> Css {
    css! {"
        display: grid;
        background: #f8f9fa;
        padding: 12px 20px;
        align-items: center;

        font-size: 0.85rem;
        font-weight: 700;
        color: #666;
        border-bottom: 1px solid #eee;
    "}
}

pub fn base_row_css() -> Css {
    css! {"
        display: grid;
        padding: 2px 10px;
        align-items: center;
        gap: 5px;

        transition: background 0.2s;
        :hover { background: #f8f8f8; }
    "}
}

pub fn edit_input_css() -> Css {
    css! {"
        width: 90%; padding: 4px 8px; border: 1px solid #ccc; border-radius: 4px;
    "}
}

pub fn read_only_col_css() -> Css {
    css! {"
        font-family: monospace; color: #888;
    "}
}

pub fn main_col_css() -> Css {
    css! {"
        font-weight: 500; color: #333;
    "}
}

pub fn normal_col_css() -> Css {
    css! {"
        color: #333; font-size: 0.9rem;
    "}
}

pub fn low_prio_col_css() -> Css {
    css! {"
        color: #666; font-size: 0.9rem;
    "}
}
