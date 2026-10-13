//! Tauri commands. They stay thin: they translate between the UI and `trove_core` / `trove_store`
//! (ADR 4). Each returns `Result<T, AppError>`, runs store calls on a blocking thread and has one
//! wrapper in `apps/desktop/src/lib/api/`; components never call a command directly.

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use trove_store::Store;
use ts_rs::TS;

use crate::error::AppError;

/// Runs `work` with the store on a blocking thread, so a slow query never stalls the UI thread.
pub async fn blocking<R, T>(
    app: &AppHandle<R>,
    work: impl FnOnce(&Store) -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError>
where
    R: Runtime,
    T: Send + 'static,
{
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || work(&app.state::<Store>()))
        .await
        .map_err(|error| AppError::internal(error.to_string()))?
}

/// Facts about this installation, for the UI to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct AppInfo {
    /// The version of the app.
    pub version: String,
    /// This installation's device id (ADR 9).
    pub device_id: String,
}

fn info(store: &Store) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        device_id: store.device_id().to_string(),
    }
}

#[tauri::command]
pub async fn app_info<R: Runtime>(app: AppHandle<R>) -> Result<AppInfo, AppError> {
    blocking(&app, |store| Ok(info(store))).await
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{
        INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
    };
    use tauri::webview::InvokeRequest;
    use tauri::{WebviewWindow, WebviewWindowBuilder};

    use crate::error::ErrorCode;

    use super::*;

    #[tauri::command]
    async fn always_fails() -> Result<(), AppError> {
        Err(AppError::new(ErrorCode::Storage, "disk full"))
    }

    /// An app on the mock runtime with a real store in a temporary data folder.
    fn app() -> (
        tauri::App<MockRuntime>,
        WebviewWindow<MockRuntime>,
        tempfile::TempDir,
    ) {
        let folder = tempfile::tempdir().unwrap();
        let store = Store::open(folder.path(), 1_760_000_000_000).unwrap();
        let app = mock_builder()
            .manage(store)
            .invoke_handler(tauri::generate_handler![app_info, always_fails])
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        (app, window, folder)
    }

    fn invoke(window: &WebviewWindow<MockRuntime>, cmd: &str) -> Result<Value, Value> {
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
            body: InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };
        get_ipc_response(window, request).map(|body| body.deserialize().unwrap())
    }

    #[test]
    fn a_command_reaches_the_ui_as_a_typed_value() {
        let (app, window, _folder) = app();
        let device_id = app.state::<Store>().device_id().to_string();

        let value = invoke(&window, "app_info").unwrap();

        assert_eq!(
            value,
            json!({ "version": env!("CARGO_PKG_VERSION"), "device_id": device_id })
        );
    }

    #[test]
    fn a_failing_command_reaches_the_ui_as_a_typed_error() {
        let (_app, window, _folder) = app();

        let error = invoke(&window, "always_fails").unwrap_err();

        assert_eq!(error, json!({ "code": "storage", "message": "disk full" }));
    }
}
