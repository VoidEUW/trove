//! The item and kind handlers on the real schema: changing rows with later operations, and refusing
//! operations they cannot apply. Every test works in its own temporary folder.

use serde_json::{Value, json};
use trove_core::{Counting, ITEM_ENTITY, Id, ItemDraft, KIND_ENTITY, KindLabel, Patch, seed_kinds};
use trove_store::{Store, StoreError};

mod common;
use common::{count, data_folder, dump};

const NOW: u64 = 1_760_000_000_000;

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

fn open(folder: &tempfile::TempDir) -> Store {
    Store::open(folder.path(), NOW).unwrap()
}

fn op_count(store: &Store) -> i64 {
    count(&store.reader().unwrap(), "ops")
}

fn new_kind(store: &Store, fields: &[(&str, Value)], at: u64) -> Id {
    let id = Id::new(at);
    store.record(KIND_ENTITY, id, patch(fields), at).unwrap();
    id
}

fn new_item(store: &Store, name: &str, kind: Id, at: u64) -> Id {
    let draft = ItemDraft {
        name: name.to_owned(),
        kind,
        quantity: 2.0,
    };
    store.add_item(draft, at).unwrap().id
}

fn refused(result: Result<impl std::fmt::Debug, StoreError>) -> String {
    match result {
        Err(StoreError::Projection(message)) => message,
        other => panic!("expected the projection to refuse, got {other:?}"),
    }
}

#[test]
fn a_kind_the_user_names_is_created_and_listed_with_the_name_as_typed() {
    let folder = data_folder();
    let store = open(&folder);

    let id = new_kind(
        &store,
        &[
            ("name", json!("Pet supplies")),
            ("counting", json!("counted")),
        ],
        NOW + 1,
    );

    let kinds = store.list_kinds().unwrap();
    assert_eq!(kinds.len(), seed_kinds().len() + 1);
    let created = kinds.last().unwrap();
    assert_eq!(created.id, id);
    assert_eq!(created.label, KindLabel::Text("Pet supplies".to_owned()));
    assert_eq!(created.counting, Counting::Counted);
    assert!(!created.container);
}

#[test]
fn renaming_a_seed_kind_replaces_its_translated_label_with_the_typed_name() {
    let folder = data_folder();
    let store = open(&folder);
    let tools = seed_kinds()[8].clone();
    assert!(matches!(tools.label, KindLabel::Key(_)));

    store
        .record(
            KIND_ENTITY,
            tools.id,
            patch(&[("name", json!("Workshop"))]),
            NOW + 1,
        )
        .unwrap();

    let renamed = &store.list_kinds().unwrap()[8];
    assert_eq!(
        (renamed.id, &renamed.label),
        (tools.id, &KindLabel::Text("Workshop".to_owned()))
    );
}

#[test]
fn a_kind_can_change_how_it_counts_hold_items_and_be_archived() {
    let folder = data_folder();
    let store = open(&folder);
    let id = new_kind(
        &store,
        &[("name", json!("Crates")), ("counting", json!("single"))],
        NOW + 1,
    );
    let listed = |store: &Store| {
        store
            .list_kinds()
            .unwrap()
            .into_iter()
            .find(|kind| kind.id == id)
    };

    store
        .record(
            KIND_ENTITY,
            id,
            patch(&[("counting", json!("counted")), ("container", json!(true))]),
            NOW + 2,
        )
        .unwrap();
    let kind = listed(&store).unwrap();
    assert_eq!((kind.counting, kind.container), (Counting::Counted, true));

    store
        .record(
            KIND_ENTITY,
            id,
            patch(&[("container", json!(false))]),
            NOW + 3,
        )
        .unwrap();
    assert!(!listed(&store).unwrap().container);

    store
        .record(
            KIND_ENTITY,
            id,
            patch(&[("archived", json!(true))]),
            NOW + 4,
        )
        .unwrap();
    assert!(listed(&store).is_none(), "an archived kind is not listed");
}

#[test]
fn the_kind_handler_refuses_what_it_cannot_apply_and_stores_nothing() {
    let folder = data_folder();
    let store = open(&folder);
    let id = new_kind(
        &store,
        &[("name", json!("Crates")), ("counting", json!("single"))],
        NOW + 1,
    );
    let before = (op_count(&store), dump(&store.reader().unwrap()));
    let record =
        |id, fields: &[(&str, Value)]| store.record(KIND_ENTITY, id, patch(fields), NOW + 9);

    assert!(refused(record(Id::new(NOW + 5), &[("counting", json!("counted"))])).contains("name"));
    assert!(refused(record(Id::new(NOW + 5), &[("name", json!("X"))])).contains("counting"));
    assert!(
        refused(record(
            Id::new(NOW + 5),
            &[("name", json!("X")), ("counting", json!("plenty"))]
        ))
        .contains("plenty")
    );
    assert!(refused(record(id, &[("counting", json!("plenty"))])).contains("plenty"));
    assert!(refused(record(id, &[("container", json!("yes"))])).contains("container"));
    assert!(refused(record(id, &[("archived", json!(1))])).contains("archived"));
    assert!(refused(record(id, &[("name", Value::Null)])).contains("name"));
    assert!(refused(record(id, &[("colour", json!("red"))])).contains("colour"));

    assert_eq!((op_count(&store), dump(&store.reader().unwrap())), before);
}

#[test]
fn later_operations_change_an_item_and_keep_when_it_was_created() {
    let folder = data_folder();
    let store = open(&folder);
    let (tools, box_kind) = (seed_kinds()[8].id, seed_kinds()[10].id);
    let id = new_item(&store, "Drill", tools, NOW + 10);

    store
        .record(
            ITEM_ENTITY,
            id,
            patch(&[
                ("name", json!("Cordless drill")),
                ("kind", json!(box_kind.to_string())),
                ("quantity", json!(3)),
            ]),
            NOW + 20,
        )
        .unwrap();

    let item = store.list_items().unwrap().remove(0);
    assert_eq!(item.name, "Cordless drill");
    assert_eq!(item.kind.id, box_kind);
    assert_eq!(item.quantity, 3);
    assert_eq!(item.created_ms, NOW + 10);
    let updated: i64 = store
        .reader()
        .unwrap()
        .query_row("SELECT updated_ms FROM items", [], |row| row.get(0))
        .unwrap();
    assert_eq!(updated, (NOW + 20) as i64);
}

#[test]
fn an_archived_item_leaves_the_list_and_can_come_back() {
    let folder = data_folder();
    let store = open(&folder);
    let tools = seed_kinds()[8].id;
    let keep = new_item(&store, "Saw", tools, NOW + 1);
    let gone = new_item(&store, "Drill", tools, NOW + 2);

    store
        .record(
            ITEM_ENTITY,
            gone,
            patch(&[("archived", json!(true))]),
            NOW + 3,
        )
        .unwrap();
    let ids = |store: &Store| -> Vec<Id> {
        store
            .list_items()
            .unwrap()
            .into_iter()
            .map(|item| item.id)
            .collect()
    };
    assert_eq!(ids(&store), vec![keep]);

    store
        .record(
            ITEM_ENTITY,
            gone,
            patch(&[("archived", json!(false))]),
            NOW + 4,
        )
        .unwrap();
    assert_eq!(ids(&store), vec![keep, gone]);
}

#[test]
fn the_item_handler_refuses_what_it_cannot_apply_and_stores_nothing() {
    let folder = data_folder();
    let store = open(&folder);
    let tools = seed_kinds()[8].id;
    let id = new_item(&store, "Drill", tools, NOW + 1);
    let before = (op_count(&store), dump(&store.reader().unwrap()));
    let record =
        |id, fields: &[(&str, Value)]| store.record(ITEM_ENTITY, id, patch(fields), NOW + 9);

    assert!(refused(record(id, &[("colour", json!("red"))])).contains("colour"));
    assert!(refused(record(id, &[("name", Value::Null)])).contains("name"));
    assert!(refused(record(id, &[("name", json!(5))])).contains("name"));
    assert!(refused(record(id, &[("quantity", json!("many"))])).contains("quantity"));
    assert!(refused(record(id, &[("archived", json!("no"))])).contains("archived"));
    // A new item needs everything a row has.
    let fresh = Id::new(NOW + 5);
    assert!(
        refused(record(
            fresh,
            &[("name", json!("Saw")), ("quantity", json!(1))]
        ))
        .contains("kind")
    );
    assert!(
        refused(record(
            fresh,
            &[("name", json!("Saw")), ("kind", json!(tools.to_string()))]
        ))
        .contains("quantity")
    );

    assert_eq!((op_count(&store), dump(&store.reader().unwrap())), before);
}

#[test]
fn kind_and_item_changes_come_back_the_same_after_a_replay() {
    let folder = data_folder();
    let store = open(&folder);
    let kind = new_kind(
        &store,
        &[("name", json!("Crates")), ("counting", json!("single"))],
        NOW + 1,
    );
    let item = new_item(&store, "Drill", kind, NOW + 2);
    store
        .record(
            KIND_ENTITY,
            seed_kinds()[8].id,
            patch(&[("name", json!("Workshop"))]),
            NOW + 3,
        )
        .unwrap();
    store
        .record(
            KIND_ENTITY,
            kind,
            patch(&[("container", json!(true))]),
            NOW + 4,
        )
        .unwrap();
    store
        .record(
            ITEM_ENTITY,
            item,
            patch(&[("quantity", json!(7)), ("archived", json!(true))]),
            NOW + 5,
        )
        .unwrap();
    let live = dump(&store.reader().unwrap());

    assert_eq!(store.replay().unwrap(), 5);

    assert_eq!(dump(&store.reader().unwrap()), live);
}
