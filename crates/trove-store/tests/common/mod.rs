//! Helpers shared by the integration tests.

use tempfile::TempDir;

/// A fresh data folder under Cargo's scratch directory (`target/tmp`), deleted when dropped. The
/// store never reads `TROVE_HOME` (ADR 8), so tests pass a folder like this to it directly.
pub fn data_folder() -> TempDir {
    tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap()
}
