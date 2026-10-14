//! Helpers for tests that run commands through Tauri's mock runtime, the way the UI calls them.

use serde_json::Value;
use tauri::ipc::{CallbackFn, Invoke, InvokeBody};
use tauri::test::{
    INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
};
use tauri::webview::InvokeRequest;
use tauri::{App, WebviewWindow, WebviewWindowBuilder};
use tempfile::TempDir;
use trove_store::Store;

/// A mock app that handles `handler`'s commands, with a real store in a temporary data folder.
/// Keep the returned folder alive for as long as the app is used.
pub fn app<H>(handler: H) -> (App<MockRuntime>, WebviewWindow<MockRuntime>, TempDir)
where
    H: Fn(Invoke<MockRuntime>) -> bool + Send + Sync + 'static,
{
    let folder = tempfile::tempdir().unwrap();
    let store = Store::open(folder.path(), 1_760_000_000_000).unwrap();
    let app = mock_builder()
        .manage(store)
        .invoke_handler(handler)
        .build(mock_context(noop_assets()))
        .unwrap();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    (app, window, folder)
}

/// Calls a command with JSON arguments. `Ok` is the value the UI would get back, `Err` the error.
pub fn invoke(window: &WebviewWindow<MockRuntime>, cmd: &str, args: Value) -> Result<Value, Value> {
    let request = InvokeRequest {
        cmd: cmd.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: if cfg!(windows) || cfg!(target_os = "android") {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        }
        .parse()
        .unwrap(),
        body: InvokeBody::Json(args),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    };
    get_ipc_response(window, request).map(|body| body.deserialize().unwrap())
}
