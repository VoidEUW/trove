//! Trove's storage: SQLite, migrations, the operation log, files and search. It implements the ports
//! defined in `trove-core` and must not depend on Tauri.

mod backup;
mod error;
pub mod migrations;
mod store;

pub use error::StoreError;
pub use store::{BACKUPS_DIR, DATABASE_FILE, Store};
