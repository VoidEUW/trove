//! The item commands: list items, list kinds and add an item (ADR 4). Each runs the store call on a
//! blocking thread; `add_item` emits `items_changed` once the transaction has committed.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use trove_core::{Counting, Id, Item, ItemDraft, Kind, KindLabel};
use ts_rs::TS;

use crate::commands::blocking;
use crate::error::{AppError, ErrorCode};
use crate::events::emit_items_changed;

/// How the items of a kind are counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CountingView {
    Single,
    Counted,
}

/// What a kind is called. A kind that ships with Trove has a message key the UI translates; a name
/// the user typed is shown as typed (ADR 15).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
#[ts(export)]
pub enum KindLabelView {
    Key(String),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct KindView {
    pub id: String,
    pub label: KindLabelView,
    pub counting: CountingView,
    /// Items of this kind can hold other items.
    pub container: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct ItemView {
    pub id: String,
    pub name: String,
    pub kind: KindView,
    pub quantity: u32,
    /// When the item was created, in Unix milliseconds.
    #[ts(type = "number")]
    pub created_ms: u64,
}

/// What the Add item form sends. Core validates it, so the form can send what the user typed.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
pub struct ItemDraftInput {
    pub name: String,
    /// The id of the chosen kind.
    pub kind: String,
    /// `null` when the field was left empty or is not a number.
    pub quantity: Option<f64>,
}

impl From<&Kind> for KindView {
    fn from(kind: &Kind) -> Self {
        Self {
            id: kind.id.to_string(),
            label: match &kind.label {
                KindLabel::Key(key) => KindLabelView::Key(key.clone()),
                KindLabel::Text(name) => KindLabelView::Text(name.clone()),
            },
            counting: match kind.counting {
                Counting::Single => CountingView::Single,
                Counting::Counted => CountingView::Counted,
            },
            container: kind.container,
        }
    }
}

impl From<&Item> for ItemView {
    fn from(item: &Item) -> Self {
        Self {
            id: item.id.to_string(),
            name: item.name.clone(),
            kind: KindView::from(&item.kind),
            quantity: item.quantity,
            created_ms: item.created_ms,
        }
    }
}

impl ItemDraftInput {
    /// A kind id that is not an id at all cannot name a kind, so it is an unknown kind.
    fn into_draft(self) -> Result<ItemDraft, AppError> {
        let kind: Id = self.kind.parse().map_err(|_| {
            AppError::new(
                ErrorCode::UnknownKind,
                format!("`{}` is not a kind id", self.kind),
            )
        })?;
        Ok(ItemDraft {
            name: self.name,
            kind,
            quantity: self.quantity.unwrap_or(f64::NAN),
        })
    }
}

#[tauri::command]
pub async fn list_items<R: Runtime>(app: AppHandle<R>) -> Result<Vec<ItemView>, AppError> {
    blocking(&app, |store| {
        Ok(store.list_items()?.iter().map(ItemView::from).collect())
    })
    .await
}

#[tauri::command]
pub async fn list_kinds<R: Runtime>(app: AppHandle<R>) -> Result<Vec<KindView>, AppError> {
    blocking(&app, |store| {
        Ok(store.list_kinds()?.iter().map(KindView::from).collect())
    })
    .await
}

/// Saves an item and then tells the views: `items_changed` is emitted after the transaction
/// committed, so a view that refetches sees it.
#[tauri::command]
pub async fn add_item<R: Runtime>(
    app: AppHandle<R>,
    draft: ItemDraftInput,
) -> Result<ItemView, AppError> {
    let draft = draft.into_draft()?;
    let (item, id) = blocking(&app, move |store| {
        let item = store.add_item(draft, crate::now_ms())?;
        Ok((ItemView::from(&item), item.id))
    })
    .await?;
    // The item is saved; a view that misses the event shows it on its next refetch.
    if let Err(error) = emit_items_changed(&app, &[id]) {
        eprintln!("could not emit items_changed: {error}");
    }
    Ok(item)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use serde_json::{Value, json};
    use tauri::Listener;
    use trove_core::seed_kinds;

    use crate::testing::{app, invoke};

    use super::*;

    fn handler() -> impl Fn(tauri::ipc::Invoke<tauri::test::MockRuntime>) -> bool + Send + Sync {
        tauri::generate_handler![list_items, list_kinds, add_item]
    }

    fn lamp_kind() -> String {
        seed_kinds()[1].id.to_string()
    }

    #[test]
    fn the_kinds_reach_the_ui_as_typed_values() {
        let (_app, window, _folder) = app(handler());

        let kinds = invoke(&window, "list_kinds", json!({})).unwrap();

        let kinds = kinds.as_array().unwrap();
        assert_eq!(kinds.len(), 11);
        assert_eq!(
            kinds[1],
            json!({
                "id": lamp_kind(),
                "label": { "type": "key", "value": "kind.furniture" },
                "counting": "single",
                "container": false
            })
        );
    }

    #[test]
    fn an_added_item_is_returned_and_then_listed() {
        let (_app, window, _folder) = app(handler());
        assert_eq!(invoke(&window, "list_items", json!({})).unwrap(), json!([]));

        let item = invoke(
            &window,
            "add_item",
            json!({ "draft": { "name": " Floor lamp ", "kind": lamp_kind(), "quantity": 2 } }),
        )
        .unwrap();

        assert_eq!(item["name"], "Floor lamp");
        assert_eq!(item["quantity"], 2);
        assert_eq!(
            item["kind"]["label"],
            json!({ "type": "key", "value": "kind.furniture" })
        );
        assert_eq!(
            invoke(&window, "list_items", json!({})).unwrap(),
            json!([item])
        );
    }

    #[test]
    fn saving_without_a_name_or_with_a_bad_quantity_returns_a_typed_error_and_stores_nothing() {
        let (_app, window, _folder) = app(handler());
        let add = |name: &str, quantity: Value| {
            invoke(
                &window,
                "add_item",
                json!({ "draft": { "name": name, "kind": lamp_kind(), "quantity": quantity } }),
            )
        };

        let code = |result: Result<Value, Value>| result.unwrap_err()["code"].clone();
        assert_eq!(code(add("", json!(1))), "name_required");
        assert_eq!(code(add("   ", json!(1))), "name_required");
        for quantity in [json!(0), json!(-2), json!(1.5), Value::Null] {
            assert_eq!(
                code(add("Lamp", quantity.clone())),
                "quantity_invalid",
                "{quantity}"
            );
        }
        let unknown = Id::seed(500).to_string();
        let result = invoke(
            &window,
            "add_item",
            json!({ "draft": { "name": "Lamp", "kind": unknown, "quantity": 1 } }),
        );
        assert_eq!(code(result), "unknown_kind");
        let result = invoke(
            &window,
            "add_item",
            json!({ "draft": { "name": "Lamp", "kind": "", "quantity": 1 } }),
        );
        assert_eq!(code(result), "unknown_kind");

        assert_eq!(invoke(&window, "list_items", json!({})).unwrap(), json!([]));
    }

    #[test]
    fn items_changed_is_emitted_after_the_item_is_committed() {
        let (app, window, _folder) = app(handler());
        let (sender, receiver) = mpsc::channel();
        app.listen(crate::events::ITEMS_CHANGED, move |event| {
            sender.send(event.payload().to_owned()).unwrap();
        });

        let item = invoke(
            &window,
            "add_item",
            json!({ "draft": { "name": "Lamp", "kind": lamp_kind(), "quantity": 1 } }),
        )
        .unwrap();

        let payload = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("no event arrived");
        assert_eq!(
            serde_json::from_str::<Value>(&payload).unwrap(),
            json!({ "ids": [item["id"]] })
        );
        // A refetch on the event sees the item.
        assert_eq!(
            invoke(&window, "list_items", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn a_failed_save_emits_no_event() {
        let (app, window, _folder) = app(handler());
        let (sender, receiver) = mpsc::channel();
        app.listen(crate::events::ITEMS_CHANGED, move |event| {
            sender.send(event.payload().to_owned()).unwrap();
        });

        invoke(
            &window,
            "add_item",
            json!({ "draft": { "name": "", "kind": lamp_kind(), "quantity": 1 } }),
        )
        .unwrap_err();

        assert!(receiver.recv_timeout(Duration::from_millis(300)).is_err());
    }
}
