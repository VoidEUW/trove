//! Tauri shell. Commands stay thin: they translate between the UI and `trove_core` / `trove_store`.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
