//! Tauri shell. Commands stay thin: they translate between the UI and `trove_core` / `trove_store`.

pub mod data_folder;

use data_folder::DataFolder;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_folder = DataFolder::from_process().expect("cannot resolve the data folder");
    data_folder.create().expect("cannot create the data folder");

    tauri::Builder::default()
        .manage(data_folder)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
