//! The store's own error type. Only the shell maps it to `AppError` (ADR 4).

use std::io;
use std::path::PathBuf;

use trove_core::ClockError;
use trove_core::Id;
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
    #[error("a stored id is not valid: {0}")]
    Id(#[from] IdError),
    #[error("a stored clock value is not valid: {0}")]
    Clock(#[from] ClockError),
    #[error("an operation's patch is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// No handler is registered for the operation's entity, so it cannot be applied.
    #[error("no projection is registered for entity `{0}`")]
    UnknownEntity(String),
    /// An operation with this id is already in the log.
    #[error("operation {0} is already in the log")]
    DuplicateOperation(Id),
    /// Older versions are read through upcasters (ADR 10), which do not exist yet.
    #[error("operation {op_id} has op_version {version}, but this Trove reads version {supported}")]
    UnsupportedOpVersion {
        op_id: Id,
        version: u32,
        supported: u32,
    },
    /// A projection refused an operation; the transaction is rolled back.
    #[error("a projection refused the operation: {0}")]
    Projection(String),
}

impl StoreError {
    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> Self {
        let path = path.into();
        move |source| Self::Io { path, source }
    }
}
