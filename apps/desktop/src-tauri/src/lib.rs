//! Tauri shell. Commands stay thin: they translate between the UI and `trove_core` / `trove_store`.

pub mod data_folder;

use std::time::{SystemTime, UNIX_EPOCH};

use data_folder::DataFolder;
use trove_store::Store;

/// Milliseconds since the Unix epoch. `trove-core` does no IO, so the shell reads the clock.
fn now_ms() -> u64 {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970");
    u64::try_from(since_epoch.as_millis()).expect("system clock is too far in the future")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_folder = DataFolder::from_process().expect("cannot resolve the data folder");
    data_folder.create().expect("cannot create the data folder");
    let store = Store::open(data_folder.root(), now_ms()).expect("cannot open the database");

    tauri::Builder::default()
        .manage(data_folder)
        .manage(store)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
