//! The embedded, forward-only migrations (ADR 8). Each file in `migrations/` is one step; the
//! database's `user_version` is the number of steps applied. A step is never edited once released.

use rusqlite_migration::{M, Migrations};

/// Every migration this version of Trove knows, oldest first.
pub fn all() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!("../migrations/0001_init.sql"))])
}
