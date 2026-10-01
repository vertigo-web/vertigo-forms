use std::rc::Rc;
use vertigo::{Computed, Resource, Value, get_driver, transaction};
use vertigo_forms::resource_table::{AsyncResult, ProcessCallback};

/// How long the server takes to reject a change, so that the change shows in the table first.
const REJECT_AFTER_MS: u32 = 1000;

/// An item of a table, with the id the server gives it.
pub trait Item: Clone + PartialEq + 'static {
    fn id(&self) -> u32;
    fn set_id(&mut self, id: u32);
}

/// Items as `ResourceTable::list` takes them: each one can change on its own.
type List<T> = Computed<Resource<Rc<Vec<Computed<Option<T>>>>>>;

/// A stand-in for the server behind a table: it keeps the items in memory and, while `rejecting`
/// is set, rejects every change.
#[derive(Clone)]
pub struct FakeServer<T: Item> {
    items: Value<Vec<Value<Option<T>>>>,
    rejecting: Value<bool>,
}

impl<T: Item> FakeServer<T> {
    pub fn new(items: Vec<T>, rejecting: &Value<bool>) -> Self {
        Self {
            items: Value::new(
                items
                    .into_iter()
                    .map(|item| Value::new(Some(item)))
                    .collect(),
            ),
            rejecting: rejecting.clone(),
        }
    }

    pub fn list(&self) -> List<T> {
        self.items.to_computed().map(|items| {
            Resource::Ready(Rc::new(
                items.iter().map(|item| item.to_computed()).collect(),
            ))
        })
    }

    /// Adds the item, with a new id.
    pub fn on_create(&self) -> ProcessCallback<T> {
        let server = self.clone();
        Rc::new(move |mut item: T| -> AsyncResult<Option<String>> {
            let server = server.clone();
            Box::pin(async move {
                if server.rejects().await {
                    return Some("Server error: the item wasn't added".to_string());
                }
                transaction(|ctx| {
                    let mut items = server.items.get(ctx);
                    item.set_id(items.len() as u32 + 10);
                    items.push(Value::new(Some(item)));
                    server.items.set(items);
                });
                None
            })
        })
    }

    /// Saves the item. The table shows the change at once, and gets the item back as it was
    /// when the server rejects it.
    pub fn on_update(&self) -> ProcessCallback<T> {
        let server = self.clone();
        Rc::new(move |item: T| -> AsyncResult<Option<String>> {
            let server = server.clone();
            Box::pin(async move {
                let Some((slot, saved)) = server.find(item.id()) else {
                    return Some("Server error: no such item".to_string());
                };
                slot.set(Some(item));
                if server.rejects().await {
                    slot.set(Some(saved));
                    return Some("Server error: the changes weren't saved".to_string());
                }
                None
            })
        })
    }

    /// Deletes the item. It's gone from the table at once, and comes back when the server
    /// rejects it.
    pub fn on_delete(&self) -> ProcessCallback<T> {
        let server = self.clone();
        Rc::new(move |item: T| -> AsyncResult<Option<String>> {
            let server = server.clone();
            Box::pin(async move {
                let Some((slot, saved)) = server.find(item.id()) else {
                    return Some("Server error: no such item".to_string());
                };
                slot.set(None);
                if server.rejects().await {
                    slot.set(Some(saved));
                    return Some("Server error: the item wasn't deleted".to_string());
                }
                None
            })
        })
    }

    fn find(&self, id: u32) -> Option<(Value<Option<T>>, T)> {
        transaction(|ctx| {
            self.items.get(ctx).into_iter().find_map(|slot| {
                let item = slot.get(ctx)?;
                (item.id() == id).then_some((slot, item))
            })
        })
    }

    /// Whether the server rejects the change. Rejecting takes it a while.
    async fn rejects(&self) -> bool {
        if !transaction(|ctx| self.rejecting.get(ctx)) {
            return false;
        }
        get_driver().sleep(REJECT_AFTER_MS).await;
        true
    }
}
