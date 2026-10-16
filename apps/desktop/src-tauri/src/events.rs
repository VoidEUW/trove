//! Events that tell the UI a write was committed (ADR 4). Views refetch on an event instead of
//! patching their own state, so the payload only names what changed.

use serde::Serialize;
use tauri::{Emitter, Runtime};
use trove_core::Id;
use ts_rs::TS;

/// Emitted after a committed write to items.
pub const ITEMS_CHANGED: &str = "items_changed";

/// Emitted after a committed write to the layout of homes, floors and rooms.
pub const LAYOUT_CHANGED: &str = "layout_changed";

/// Payload of [`ITEMS_CHANGED`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct ItemsChanged {
    /// The items that were written.
    pub ids: Vec<String>,
}

/// Payload of [`LAYOUT_CHANGED`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct LayoutChanged {
    /// The homes, floors or rooms that were written.
    pub ids: Vec<String>,
}

fn strings(ids: &[Id]) -> Vec<String> {
    ids.iter().map(Id::to_string).collect()
}

/// Call after the transaction that wrote items has committed.
pub fn emit_items_changed<R: Runtime>(app: &impl Emitter<R>, ids: &[Id]) -> tauri::Result<()> {
    app.emit(ITEMS_CHANGED, ItemsChanged { ids: strings(ids) })
}

/// Call after the transaction that wrote the layout has committed.
pub fn emit_layout_changed<R: Runtime>(app: &impl Emitter<R>, ids: &[Id]) -> tauri::Result<()> {
    app.emit(LAYOUT_CHANGED, LayoutChanged { ids: strings(ids) })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use serde_json::json;
    use tauri::Listener;

    use super::*;

    /// Emits with `emit` and returns the JSON payload a subscribed listener receives.
    fn received(
        event: &str,
        emit: impl FnOnce(&tauri::AppHandle<tauri::test::MockRuntime>),
    ) -> serde_json::Value {
        let app = tauri::test::mock_app();
        let (sender, receiver) = mpsc::channel();
        app.listen(event, move |event| {
            sender.send(event.payload().to_owned()).unwrap();
        });
        emit(app.handle());
        let payload = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("no event arrived");
        serde_json::from_str(&payload).unwrap()
    }

    #[test]
    fn names_are_the_ones_the_ui_listens_to() {
        assert_eq!(ITEMS_CHANGED, "items_changed");
        assert_eq!(LAYOUT_CHANGED, "layout_changed");
    }

    #[test]
    fn a_subscribed_listener_gets_the_ids_of_what_was_written() {
        let (a, b) = (Id::new(1), Id::new(2));
        let payload = received(ITEMS_CHANGED, |app| {
            emit_items_changed(app, &[a, b]).unwrap()
        });
        assert_eq!(payload, json!({ "ids": [a.to_string(), b.to_string()] }));

        let room = Id::new(3);
        let payload = received(LAYOUT_CHANGED, |app| {
            emit_layout_changed(app, &[room]).unwrap()
        });
        assert_eq!(payload, json!({ "ids": [room.to_string()] }));
    }
}
