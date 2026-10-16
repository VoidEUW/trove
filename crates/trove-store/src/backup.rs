//! Snapshots taken with SQLite's backup API before a migration changes an existing database
//! (ADR 8: downgrading is impossible, so the way back is a backup).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::Connection;
use rusqlite::backup::Backup;

use crate::error::StoreError;

const PAGES_PER_STEP: i32 = 256;

/// Copies the database behind `conn` into `dir` as `trove-v<from>-<now_ms>.db`, where `from` is the
/// version it is about to leave. An existing backup is never overwritten.
pub(crate) fn before_migration(
    conn: &Connection,
    dir: &Path,
    from: usize,
    now_ms: u64,
) -> Result<PathBuf, StoreError> {
    fs::create_dir_all(dir).map_err(StoreError::io(dir))?;
    let path = dir.join(format!("trove-v{from}-{now_ms}.db"));
    if path.exists() {
        let exists = io::Error::new(io::ErrorKind::AlreadyExists, "backup already exists");
        return Err(StoreError::io(&path)(exists));
    }

    let mut copy = Connection::open(&path)?;
    Backup::new(conn, &mut copy)?.run_to_completion(PAGES_PER_STEP, Duration::ZERO, None)?;
    Ok(path)
}
