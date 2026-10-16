//! Trove's storage: SQLite, migrations, the operation log, files and search. It implements the ports
//! defined in `trove-core` and must not depend on Tauri.

mod backup;
mod error;
mod items;
pub mod migrations;
mod oplog;
mod projection;
mod store;

pub use error::StoreError;
pub use items::{ItemProjection, KindProjection};
pub use projection::Projection;
pub use store::{BACKUPS_DIR, DATABASE_FILE, Store};
