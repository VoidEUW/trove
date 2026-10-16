//! Helpers shared by the integration tests. Each test file uses some of them.

#![allow(dead_code)]

use rusqlite::Connection;
use tempfile::TempDir;

/// A fresh data folder under Cargo's scratch directory (`target/tmp`), deleted when dropped. The
/// store never reads `TROVE_HOME` (ADR 8), so tests pass a folder like this to it directly.
pub fn data_folder() -> TempDir {
    tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap()
}

pub fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

/// Everything about the projection tables, in a comparable form.
pub fn dump(conn: &Connection) -> Vec<String> {
    let mut rows = Vec::new();
    for sql in [
        "SELECT id || '|' || coalesce(label_key, '') || '|' || coalesce(name, '') || '|' || counting || '|' || container || '|' || archived FROM kinds ORDER BY id",
        "SELECT id || '|' || name || '|' || kind_id || '|' || quantity || '|' || created_ms || '|' || updated_ms || '|' || archived FROM items ORDER BY created_ms, id",
    ] {
        let mut statement = conn.prepare(sql).unwrap();
        rows.extend(
            statement
                .query_map([], |row| row.get::<_, String>(0))
                .unwrap()
                .map(Result::unwrap),
        );
    }
    rows
}
