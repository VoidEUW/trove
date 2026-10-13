//! Opening, migrating and backing up the database. Every test works in its own temporary folder
//! under Cargo's scratch directory (`target/tmp`) and never touches real data.

use std::fs;
use std::path::Path;

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};
use trove_store::{BACKUPS_DIR, DATABASE_FILE, Store, StoreError};

mod common;
use common::data_folder;

fn user_version(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap()
}

fn tables(conn: &Connection) -> Vec<String> {
    let mut statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap();
    statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn backups(root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root.join(BACKUPS_DIR)) else {
        return Vec::new();
    };
    let mut names: Vec<_> = entries
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

const INIT: &str = include_str!("../migrations/0001_init.sql");

/// Two releases of the schema on top of the real first migration, so an upgrade has something to do.
fn release_1() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(INIT),
        M::up("CREATE TABLE note (text TEXT NOT NULL);"),
    ])
}

fn release_2() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(INIT),
        M::up("CREATE TABLE note (text TEXT NOT NULL);"),
        M::up("CREATE TABLE tag (label TEXT NOT NULL);"),
    ])
}

#[test]
fn a_new_data_folder_gets_the_database_with_the_log_items_and_kinds() {
    let folder = data_folder();
    let root = folder.path().join("trove");

    let store = Store::open(&root, 1_760_000_000_000).unwrap();

    assert!(root.join(DATABASE_FILE).is_file());
    assert_eq!(store.database(), root.join(DATABASE_FILE));
    let reader = store.reader().unwrap();
    assert_eq!(tables(&reader), ["items", "kinds", "meta", "ops"]);
    assert_eq!(user_version(&reader), 2);
    assert!(
        backups(&root).is_empty(),
        "a brand-new database has nothing to back up"
    );
}

#[test]
fn connections_use_wal_and_foreign_keys() {
    let folder = data_folder();
    let store = Store::open(folder.path(), 1_000).unwrap();

    let journal: String = store
        .reader()
        .unwrap()
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();
    assert_eq!(journal, "wal");

    let foreign_keys: i64 = store
        .with_writer(|conn| Ok(conn.pragma_query_value(None, "foreign_keys", |row| row.get(0))?))
        .unwrap();
    assert_eq!(foreign_keys, 1);
}

#[test]
fn the_log_rejects_a_patch_that_is_not_json() {
    let folder = data_folder();
    let store = Store::open(folder.path(), 1_000).unwrap();
    let insert = |patch: &str| {
        store.with_writer(|conn| {
            Ok(conn.execute(
                "INSERT INTO ops VALUES ('a', 'd', 1, 1, 'item', 'e', ?1)",
                [patch],
            )?)
        })
    };
    assert!(insert("not json").is_err());
    assert_eq!(insert(r#"{"name":"Lamp","note":null}"#).unwrap(), 1);
}

#[test]
fn a_database_from_a_newer_trove_is_refused_and_left_untouched() {
    let folder = data_folder();
    let database = folder.path().join(DATABASE_FILE);
    {
        let conn = Connection::open(&database).unwrap();
        conn.execute_batch("CREATE TABLE future (x INTEGER); PRAGMA user_version = 99;")
            .unwrap();
    }
    let before = fs::read(&database).unwrap();

    let error = Store::open(folder.path(), 1_000).unwrap_err();

    assert!(
        matches!(
            error,
            StoreError::DatabaseTooNew {
                found: 99,
                supported: 2
            }
        ),
        "got {error:?}"
    );
    assert_eq!(fs::read(&database).unwrap(), before);
    assert!(backups(folder.path()).is_empty());
}

#[test]
fn an_upgrade_is_backed_up_first_and_an_up_to_date_open_is_not() {
    let folder = data_folder();
    let root = folder.path();

    let old = Store::open_with(root, 1_000, &release_1()).unwrap();
    old.with_writer(|conn| Ok(conn.execute("INSERT INTO note VALUES ('kept')", [])?))
        .unwrap();
    assert!(
        backups(root).is_empty(),
        "creating the database is not an upgrade"
    );
    drop(old);

    let new = Store::open_with(root, 2_000, &release_2()).unwrap();
    assert_eq!(backups(root), ["trove-v2-2000.db"]);
    let live = new.reader().unwrap();
    assert_eq!(user_version(&live), 3);
    assert_eq!(tables(&live), ["meta", "note", "ops", "tag"]);

    // The backup is a complete database as it was before the upgrade.
    {
        let backup = Connection::open(root.join(BACKUPS_DIR).join("trove-v2-2000.db")).unwrap();
        assert_eq!(user_version(&backup), 2);
        assert_eq!(tables(&backup), ["meta", "note", "ops"]);
        let kept: String = backup
            .query_row("SELECT text FROM note", [], |row| row.get(0))
            .unwrap();
        assert_eq!(kept, "kept");
    }
    drop(new);

    Store::open_with(root, 3_000, &release_2()).unwrap();
    assert_eq!(
        backups(root),
        ["trove-v2-2000.db"],
        "nothing to migrate, so no new backup"
    );
}

#[test]
fn the_device_id_is_created_once_and_stays_the_same() {
    let folder = data_folder();

    let first = Store::open(folder.path(), 1_760_000_000_000).unwrap();
    let id = first.device_id();
    assert_eq!(id.timestamp_ms(), 1_760_000_000_000);
    drop(first);

    let second = Store::open(folder.path(), 1_770_000_000_000).unwrap();
    assert_eq!(second.device_id(), id);

    let stored: String = second
        .reader()
        .unwrap()
        .query_row(
            "SELECT value FROM meta WHERE key = 'device_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, id.to_string());
}
