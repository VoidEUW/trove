//! The per-entity handlers that turn operations into rows (ADR 10, ADR 16). The entity tables are
//! a projection of the log: applying every operation in order must give the same rows as the live
//! database, so a handler may use nothing but the operation and its own tables.

use std::fmt::Debug;

use rusqlite::Transaction;
use trove_core::Operation;

use crate::error::StoreError;

/// Applies the operations of one entity type to the tables it owns. Handlers run inside the
/// transaction that appends the operation: if one fails, the operation is not stored either.
pub trait Projection: Debug + Send + Sync {
    /// The entity type this handler owns, as written in `Operation::entity`, such as `"item"`.
    fn entity(&self) -> &str;

    /// Applies the operation's patch: the first operation for an entity id creates its row, a
    /// patch value sets a field and `null` clears it.
    fn apply(&self, tx: &Transaction<'_>, op: &Operation) -> Result<(), StoreError>;

    /// Empties every table the handler owns, ready for a replay.
    fn clear(&self, tx: &Transaction<'_>) -> Result<(), StoreError>;
}
