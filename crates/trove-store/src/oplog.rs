//! The `ops` table: appending operations and reading them back in replay order.

use rusqlite::{Connection, Row};
use trove_core::{Hlc, OP_VERSION, Operation, Patch};

use crate::error::StoreError;

/// Every operation in the order replay applies them: `(hlc, device_id)` as in ADR 9, with the
/// operation id as the final tie-break so the order is total.
pub(crate) const ORDERED: &str = "SELECT op_id, device_id, hlc, op_version, entity, entity_id, patch \
     FROM ops ORDER BY hlc, device_id, op_id";

/// Appends `op`. An id that is already in the log is refused, not overwritten.
pub(crate) fn append(conn: &Connection, op: &Operation) -> Result<(), StoreError> {
    let known: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM ops WHERE op_id = ?1)",
        [op.op_id.to_string()],
        |row| row.get(0),
    )?;
    if known {
        return Err(StoreError::DuplicateOperation(op.op_id));
    }
    conn.execute(
        "INSERT INTO ops (op_id, device_id, hlc, op_version, entity, entity_id, patch) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            op.op_id.to_string(),
            op.device_id.to_string(),
            op.hlc.as_i64(),
            op.op_version,
            &op.entity,
            op.entity_id.to_string(),
            serde_json::to_string(&op.patch)?,
        ),
    )?;
    Ok(())
}

/// Reads one row of [`ORDERED`].
pub(crate) fn operation_from_row(row: &Row<'_>) -> Result<Operation, StoreError> {
    let op = Operation {
        op_id: row.get::<_, String>(0)?.parse()?,
        device_id: row.get::<_, String>(1)?.parse()?,
        hlc: Hlc::from_i64(row.get(2)?)?,
        op_version: row.get(3)?,
        entity: row.get(4)?,
        entity_id: row.get::<_, String>(5)?.parse()?,
        patch: serde_json::from_str::<Patch>(&row.get::<_, String>(6)?)?,
    };
    check_version(&op)?;
    Ok(op)
}

/// Only the current format is read until upcasters exist for older ones (ADR 10).
pub(crate) fn check_version(op: &Operation) -> Result<(), StoreError> {
    if op.op_version != OP_VERSION {
        return Err(StoreError::UnsupportedOpVersion {
            op_id: op.op_id,
            version: op.op_version,
            supported: OP_VERSION,
        });
    }
    Ok(())
}

/// The newest timestamp in the log, or [`Hlc::ZERO`] for an empty one.
pub(crate) fn latest_hlc(conn: &Connection) -> Result<Hlc, StoreError> {
    let latest: Option<i64> = conn.query_row("SELECT MAX(hlc) FROM ops", [], |row| row.get(0))?;
    Ok(latest.map_or(Ok(Hlc::ZERO), Hlc::from_i64)?)
}
