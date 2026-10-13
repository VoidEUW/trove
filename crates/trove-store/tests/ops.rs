//! Writing operations and replaying them. The tests use a made-up entity, `thing`, with its own
//! table: the first real entity (items) arrives with a later issue.

use std::path::Path;

use rusqlite::{Connection, Transaction};
use rusqlite_migration::{M, Migrations};
use serde_json::{Value, json};
use trove_core::{Hlc, Id, Operation, Patch};
use trove_store::{Projection, Store, StoreError};

mod common;
use common::data_folder;

const NOW: u64 = 1_760_000_000_000;

/// A test entity: `name` and `qty` set or cleared by a patch. Any other field is refused, so a test can
/// make the projection fail after earlier fields of the same patch were applied.
#[derive(Debug)]
struct Things;

impl Projection for Things {
    fn entity(&self) -> &str {
        "thing"
    }

    fn apply(&self, tx: &Transaction<'_>, op: &Operation) -> Result<(), StoreError> {
        let id = op.entity_id.to_string();
        tx.execute("INSERT OR IGNORE INTO thing (id) VALUES (?1)", [&id])?;
        for (field, value) in op.patch.iter() {
            match field {
                "name" => tx.execute(
                    "UPDATE thing SET name = ?2 WHERE id = ?1",
                    (&id, value.and_then(Value::as_str)),
                )?,
                "qty" => tx.execute(
                    "UPDATE thing SET qty = ?2 WHERE id = ?1",
                    (&id, value.and_then(Value::as_i64)),
                )?,
                other => return Err(StoreError::Projection(format!("cannot set `{other}`"))),
            };
        }
        Ok(())
    }

    fn clear(&self, tx: &Transaction<'_>) -> Result<(), StoreError> {
        tx.execute("DELETE FROM thing", [])?;
        Ok(())
    }
}

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("../migrations/0001_init.sql")),
        M::up("CREATE TABLE thing (id TEXT PRIMARY KEY, name TEXT, qty INTEGER) STRICT;"),
    ])
}

fn open(root: &Path, now_ms: u64) -> Store {
    let mut store = Store::open_with(root, now_ms, &migrations()).unwrap();
    store.register(Things).unwrap();
    store
}

type Row = (String, Option<String>, Option<i64>);

fn things(conn: &Connection) -> Vec<Row> {
    let mut statement = conn
        .prepare("SELECT id, name, qty FROM thing ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn op_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM ops", [], |row| row.get(0))
        .unwrap()
}

fn patch(fields: &[(&str, Value)]) -> Patch {
    let mut patch = Patch::new();
    for (field, value) in fields {
        if value.is_null() {
            patch.clear(*field).unwrap();
        } else {
            patch.set(*field, value.clone()).unwrap();
        }
    }
    patch
}

/// Some history: two things, changed, cleared and changed again.
fn write_history(store: &Store) -> (Id, Id) {
    let (lamp, chair) = (Id::new(NOW), Id::new(NOW));
    let at = |step: u64| NOW + step * 10;
    store
        .record(
            "thing",
            lamp,
            patch(&[("name", json!("Lamp")), ("qty", json!(1))]),
            at(1),
        )
        .unwrap();
    store
        .record("thing", chair, patch(&[("name", json!("Chair"))]), at(2))
        .unwrap();
    store
        .record("thing", lamp, patch(&[("name", json!("Desk lamp"))]), at(3))
        .unwrap();
    store
        .record("thing", lamp, patch(&[("qty", json!(3))]), at(3))
        .unwrap();
    store
        .record(
            "thing",
            chair,
            patch(&[("name", Value::Null), ("qty", json!(4))]),
            at(4),
        )
        .unwrap();
    store
        .record("thing", lamp, patch(&[("qty", Value::Null)]), at(5))
        .unwrap();
    (lamp, chair)
}

#[test]
fn a_write_appends_the_operation_and_updates_the_projection() {
    let folder = data_folder();
    let store = open(folder.path(), NOW);
    let id = Id::new(NOW);

    let op = store
        .record(
            "thing",
            id,
            patch(&[("name", json!("Lamp")), ("qty", json!(2))]),
            NOW,
        )
        .unwrap();

    assert_eq!(
        (op.device_id, op.entity.as_str(), op.entity_id),
        (store.device_id(), "thing", id)
    );
    assert_eq!(op.hlc, Hlc::new(NOW, 0).unwrap());
    let reader = store.reader().unwrap();
    assert_eq!(
        things(&reader),
        [(id.to_string(), Some("Lamp".to_owned()), Some(2))]
    );
    let (hlc, patch_text): (i64, String) = reader
        .query_row(
            "SELECT hlc, patch FROM ops WHERE op_id = ?1",
            [op.op_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(hlc, op.hlc.as_i64());
    assert_eq!(
        serde_json::from_str::<Value>(&patch_text).unwrap(),
        json!({"name": "Lamp", "qty": 2})
    );

    store
        .record("thing", id, patch(&[("name", Value::Null)]), NOW + 1)
        .unwrap();
    assert_eq!(
        things(&reader),
        [(id.to_string(), None, Some(2))],
        "null clears a field"
    );
    assert_eq!(op_count(&reader), 2);
}

#[test]
fn if_the_projection_fails_the_operation_is_not_stored_either() {
    let folder = data_folder();
    let store = open(folder.path(), NOW);
    let id = Id::new(NOW);
    store
        .record("thing", id, patch(&[("name", json!("Lamp"))]), NOW)
        .unwrap();

    // `name` is applied before `poison` fails, so its change must be rolled back as well.
    let failed = store.record(
        "thing",
        id,
        patch(&[("name", json!("Gone")), ("poison", json!(true))]),
        NOW + 1,
    );

    assert!(
        matches!(failed, Err(StoreError::Projection(_))),
        "got {failed:?}"
    );
    let reader = store.reader().unwrap();
    assert_eq!(
        things(&reader),
        [(id.to_string(), Some("Lamp".to_owned()), None)]
    );
    assert_eq!(op_count(&reader), 1);

    // The failed write did not use up a clock value.
    let next = store
        .record("thing", id, patch(&[("qty", json!(1))]), NOW)
        .unwrap();
    assert_eq!(next.hlc, Hlc::new(NOW, 1).unwrap());
}

#[test]
fn if_the_log_refuses_the_operation_the_projection_is_untouched() {
    let folder = data_folder();
    let store = open(folder.path(), NOW);
    let id = Id::new(NOW);
    let op = store
        .record("thing", id, patch(&[("name", json!("Lamp"))]), NOW)
        .unwrap();

    let again = Operation {
        patch: patch(&[("name", json!("Other"))]),
        ..op.clone()
    };

    assert!(
        matches!(store.apply(&op), Err(StoreError::DuplicateOperation(dup)) if dup == op.op_id)
    );
    assert!(matches!(
        store.apply(&again),
        Err(StoreError::DuplicateOperation(_))
    ));
    let reader = store.reader().unwrap();
    assert_eq!(
        things(&reader),
        [(id.to_string(), Some("Lamp".to_owned()), None)]
    );
    assert_eq!(op_count(&reader), 1);
}

#[test]
fn an_operation_nobody_can_apply_is_refused_and_not_stored() {
    let folder = data_folder();
    let store = open(folder.path(), NOW);
    let hlc = Hlc::new(NOW, 0).unwrap();
    let ghost = Operation::new(
        store.device_id(),
        hlc,
        "ghost",
        Id::new(NOW),
        patch(&[("a", json!(1))]),
    );
    let future = Operation {
        op_version: 2,
        ..Operation::new(store.device_id(), hlc, "thing", Id::new(NOW), Patch::new())
    };

    assert!(
        matches!(store.apply(&ghost), Err(StoreError::UnknownEntity(entity)) if entity == "ghost")
    );
    assert!(matches!(
        store.apply(&future),
        Err(StoreError::UnsupportedOpVersion { version: 2, .. })
    ));
    assert_eq!(op_count(&store.reader().unwrap()), 0);
}

#[test]
fn one_entity_cannot_have_two_projections() {
    let folder = data_folder();
    let mut store = open(folder.path(), NOW);
    assert!(matches!(
        store.register(Things),
        Err(StoreError::Projection(_))
    ));
}

#[test]
fn replaying_the_log_into_an_empty_database_gives_the_live_tables() {
    let live_folder = data_folder();
    let live = open(live_folder.path(), NOW);
    write_history(&live);
    let expected = things(&live.reader().unwrap());
    assert_eq!(expected.len(), 2);

    let empty_folder = data_folder();
    let empty = open(empty_folder.path(), NOW);
    assert!(things(&empty.reader().unwrap()).is_empty());
    // Copy the log newest first, so the result can only come from replay's ordering.
    empty
        .with_writer(|conn| {
            conn.execute(
                "ATTACH DATABASE ?1 AS live",
                [live.database().to_str().unwrap()],
            )?;
            conn.execute(
                "INSERT INTO ops SELECT * FROM live.ops ORDER BY hlc DESC",
                [],
            )?;
            conn.execute_batch("DETACH DATABASE live")?;
            Ok(())
        })
        .unwrap();

    assert_eq!(empty.replay().unwrap(), 6);
    assert_eq!(things(&empty.reader().unwrap()), expected);
}

#[test]
fn replay_repairs_a_projection_that_was_damaged() {
    let folder = data_folder();
    let store = open(folder.path(), NOW);
    write_history(&store);
    let expected = things(&store.reader().unwrap());

    store
        .with_writer(|conn| {
            conn.execute("UPDATE thing SET qty = 999", [])?;
            conn.execute("DELETE FROM thing WHERE name IS NOT NULL", [])?;
            Ok(())
        })
        .unwrap();
    assert_ne!(things(&store.reader().unwrap()), expected);

    store.replay().unwrap();
    assert_eq!(things(&store.reader().unwrap()), expected);
    assert_eq!(
        op_count(&store.reader().unwrap()),
        6,
        "replay never touches the log"
    );
}

#[test]
fn a_replay_that_fails_leaves_the_projection_as_it_was() {
    let folder = data_folder();
    let store = open(folder.path(), NOW);
    write_history(&store);
    let before = things(&store.reader().unwrap());

    store
        .with_writer(|conn| {
            conn.execute(
                "INSERT INTO ops VALUES (?1, ?2, ?3, 1, 'ghost', ?1, '{}')",
                (
                    Id::new(NOW).to_string(),
                    store.device_id().to_string(),
                    Hlc::new(NOW + 1_000, 0).unwrap().as_i64(),
                ),
            )?;
            Ok(())
        })
        .unwrap();

    assert!(matches!(store.replay(), Err(StoreError::UnknownEntity(_))));
    assert_eq!(things(&store.reader().unwrap()), before);
}

#[test]
fn the_clock_continues_after_a_reopen_even_if_the_wall_clock_went_back() {
    let folder = data_folder();
    let store = open(folder.path(), NOW);
    let id = Id::new(NOW);
    store
        .record("thing", id, patch(&[("qty", json!(1))]), NOW)
        .unwrap();
    let last = store
        .record("thing", id, patch(&[("qty", json!(2))]), NOW)
        .unwrap();
    assert_eq!(last.hlc, Hlc::new(NOW, 1).unwrap());
    drop(store);

    let store = open(folder.path(), NOW - 60_000);
    let next = store
        .record("thing", id, patch(&[("qty", json!(3))]), NOW - 60_000)
        .unwrap();
    assert_eq!(next.hlc, Hlc::new(NOW, 2).unwrap());
}
