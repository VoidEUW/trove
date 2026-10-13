//! Items and kinds on the real schema: adding, listing, the seed kinds and replay. Every test works
//! in its own temporary folder under Cargo's scratch directory.

use rusqlite::Connection;
use trove_core::{Id, Item, ItemDraft, ItemError, KindLabel, seed_kinds};
use trove_store::{Store, StoreError};

mod common;
use common::data_folder;

const NOW: u64 = 1_760_000_000_000;

fn draft(name: &str, kind: Id, quantity: f64) -> ItemDraft {
    ItemDraft {
        name: name.to_owned(),
        kind,
        quantity,
    }
}

fn lamp_kind() -> Id {
    seed_kinds()[1].id
}

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

/// Everything about the projection tables, in a comparable form.
fn dump(conn: &Connection) -> Vec<String> {
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

#[test]
fn the_seed_kinds_in_the_database_are_the_ones_core_lists() {
    let folder = data_folder();
    let store = Store::open(folder.path(), NOW).unwrap();

    let stored = store.list_kinds().unwrap();

    assert_eq!(stored, seed_kinds());
    assert_eq!(stored.len(), 11);
    assert!(
        stored
            .iter()
            .all(|kind| matches!(kind.label, KindLabel::Key(_)))
    );
}

#[test]
fn an_added_item_is_stored_and_listed_with_its_kind() {
    let folder = data_folder();
    let store = Store::open(folder.path(), NOW).unwrap();
    let kind = lamp_kind();

    let item = store
        .add_item(draft("  Floor lamp ", kind, 2.0), NOW + 5)
        .unwrap();

    assert_eq!(
        (item.name.as_str(), item.quantity, item.created_ms),
        ("Floor lamp", 2, NOW + 5)
    );
    assert_eq!(item.kind.id, kind);
    assert_eq!(store.list_items().unwrap(), vec![item.clone()]);
    let reader = store.reader().unwrap();
    assert_eq!(count(&reader, "items"), 1);
    assert_eq!(count(&reader, "ops"), 1, "one operation per item");
}

#[test]
fn items_are_listed_oldest_first() {
    let folder = data_folder();
    let store = Store::open(folder.path(), NOW).unwrap();
    for (step, name) in ["First", "Second", "Third"].into_iter().enumerate() {
        store
            .add_item(draft(name, lamp_kind(), 1.0), NOW + step as u64)
            .unwrap();
    }

    let names: Vec<String> = store
        .list_items()
        .unwrap()
        .into_iter()
        .map(|item| item.name)
        .collect();

    assert_eq!(names, ["First", "Second", "Third"]);
}

#[test]
fn an_invalid_draft_stores_nothing() {
    let folder = data_folder();
    let store = Store::open(folder.path(), NOW).unwrap();

    let blank = store.add_item(draft("   ", lamp_kind(), 1.0), NOW);
    assert!(
        matches!(blank, Err(StoreError::Item(ItemError::NameRequired))),
        "{blank:?}"
    );
    for quantity in [0.0, -3.0, 1.5, f64::NAN] {
        let bad = store.add_item(draft("Lamp", lamp_kind(), quantity), NOW);
        assert!(
            matches!(bad, Err(StoreError::Item(ItemError::QuantityInvalid(_)))),
            "{quantity}: {bad:?}"
        );
    }
    let unknown = Id::seed(500);
    let missing = store.add_item(draft("Lamp", unknown, 1.0), NOW);
    assert!(
        matches!(missing, Err(StoreError::UnknownKind(id)) if id == unknown),
        "{missing:?}"
    );

    let reader = store.reader().unwrap();
    assert_eq!((count(&reader, "items"), count(&reader, "ops")), (0, 0));
}

#[test]
fn replaying_the_log_into_an_empty_database_gives_the_live_tables() {
    let live_folder = data_folder();
    let live = Store::open(live_folder.path(), NOW).unwrap();
    for (step, (name, quantity)) in [("Lamp", 1.0), ("Screws", 40.0), ("Mixer", 1.0)]
        .into_iter()
        .enumerate()
    {
        let kind = seed_kinds()[step].id;
        live.add_item(draft(name, kind, quantity), NOW + step as u64 * 10)
            .unwrap();
    }
    let expected = dump(&live.reader().unwrap());
    assert_eq!(live.list_items().unwrap().len(), 3);

    let empty_folder = data_folder();
    let empty = Store::open(empty_folder.path(), NOW).unwrap();
    assert!(empty.list_items().unwrap().is_empty());
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

    assert_eq!(empty.replay().unwrap(), 3);

    assert_eq!(dump(&empty.reader().unwrap()), expected);
    assert_eq!(
        empty.list_kinds().unwrap(),
        seed_kinds(),
        "the seed kinds survive a replay"
    );
}

#[test]
fn replay_repairs_damaged_items_and_kinds_and_keeps_the_log() {
    let folder = data_folder();
    let store = Store::open(folder.path(), NOW).unwrap();
    let first = store
        .add_item(draft("Lamp", lamp_kind(), 2.0), NOW)
        .unwrap();
    store
        .add_item(draft("Chair", lamp_kind(), 4.0), NOW + 1)
        .unwrap();
    let expected: Vec<Item> = store.list_items().unwrap();

    store
        .with_writer(|conn| {
            conn.execute("UPDATE items SET quantity = 99", [])?;
            conn.execute("DELETE FROM items WHERE name = 'Chair'", [])?;
            conn.execute("UPDATE kinds SET counting = 'counted', archived = 1", [])?;
            Ok(())
        })
        .unwrap();
    assert_ne!(store.list_items().unwrap(), expected);
    assert!(store.list_kinds().unwrap().is_empty());

    assert_eq!(store.replay().unwrap(), 2);
    assert_eq!(store.list_kinds().unwrap(), seed_kinds());

    assert_eq!(store.list_items().unwrap(), expected);
    assert_eq!(store.list_items().unwrap()[0].id, first.id);
    assert_eq!(count(&store.reader().unwrap(), "ops"), 2);
}

#[test]
fn an_item_added_before_a_restart_is_listed_after_it() {
    let folder = data_folder();
    let store = Store::open(folder.path(), NOW).unwrap();
    let saved = store
        .add_item(draft("Lamp", lamp_kind(), 3.0), NOW)
        .unwrap();
    drop(store);

    let reopened = Store::open(folder.path(), NOW + 60_000).unwrap();

    assert_eq!(reopened.list_items().unwrap(), vec![saved]);
}

#[test]
fn a_kind_that_items_refer_to_cannot_be_deleted() {
    let folder = data_folder();
    let store = Store::open(folder.path(), NOW).unwrap();
    store
        .add_item(draft("Lamp", lamp_kind(), 1.0), NOW)
        .unwrap();

    let result = store.with_writer(|conn| {
        Ok(conn.execute("DELETE FROM kinds WHERE id = ?1", [lamp_kind().to_string()])?)
    });

    assert!(result.is_err(), "foreign keys are on");
    assert_eq!(store.list_kinds().unwrap().len(), 11);
}
