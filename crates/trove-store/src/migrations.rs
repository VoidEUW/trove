//! The embedded, forward-only migrations (ADR 8). Each file in `migrations/` is one step; the
//! database's `user_version` is the number of steps applied. A step is never edited once released.

use rusqlite_migration::{M, Migrations};

/// Every migration this version of Trove knows, oldest first.
pub fn all() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("../migrations/0001_init.sql")),
        M::up(concat!(
            include_str!("../migrations/0002_items_kinds.sql"),
            include_str!("../migrations/seed_kinds.sql")
        )),
    ])
}

/// The seed kinds, for the handler that puts them back when the projection is rebuilt.
pub(crate) const SEED_KINDS_SQL: &str = include_str!("../migrations/seed_kinds.sql");
