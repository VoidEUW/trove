//! The store's own error type. Only the shell maps it to `AppError` (ADR 4).

use std::io;
use std::path::PathBuf;

use trove_core::id::IdError;

/// Why the store could not open, migrate or use the database.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The database was written by a newer Trove. It is left untouched.
    #[error(
        "the database is at version {found}, newer than the version {supported} this Trove knows; update Trove"
    )]
    DatabaseTooNew { found: usize, supported: usize },
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("migration failed: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("cannot use `{}`: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("the stored device id is not valid: {0}")]
    DeviceId(#[from] IdError),
}

impl StoreError {
    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> Self {
        let path = path.into();
        move |source| Self::Io { path, source }
    }
}
