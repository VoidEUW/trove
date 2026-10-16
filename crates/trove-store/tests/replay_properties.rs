//! Replay on the real schema (ADR 10, ADR 17 tier 1): for any history of item and kind changes,
//! rebuilding the projection from the log gives the live tables, whatever order the log was copied in.
//! Cases are bounded; inputs proptest finds are kept in `replay_properties.proptest-regressions`.

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};
use trove_core::{ITEM_ENTITY, Id, ItemDraft, KIND_ENTITY, Patch, seed_kinds};
use trove_store::{Store, StoreError};

mod common;
use common::{data_folder, dump};

const NOW: u64 = 1_760_000_000_000;

/// Inputs that failed once are kept next to this file and replayed first.
fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/replay_properties.proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}

#[derive(Clone, Debug)]
enum Step {
    AddItem {
        name: String,
        kind: usize,
        quantity: u32,
    },
    RenameItem {
        item: usize,
        name: String,
    },
    SetQuantity {
        item: usize,
        quantity: u32,
    },
    MoveItem {
        item: usize,
        kind: usize,
    },
    ArchiveItem {
        item: usize,
        archived: bool,
    },
    AddKind {
        name: String,
        counted: bool,
    },
    RenameKind {
        kind: usize,
        name: String,
    },
    SetContainer {
        kind: usize,
        container: bool,
    },
    ArchiveKind {
        kind: usize,
    },
}

fn name() -> impl Strategy<Value = String> {
    "[a-z\u{e9}\u{6f22}\u{1f389}][a-z \u{e9}\u{6f22}\u{1f389}]{0,10}"
}

fn step() -> impl Strategy<Value = (Step, u64)> {
    let step = prop_oneof![
        (name(), any::<usize>(), 1..100u32).prop_map(|(name, kind, quantity)| Step::AddItem {
            name,
            kind,
            quantity
        }),
        (any::<usize>(), name()).prop_map(|(item, name)| Step::RenameItem { item, name }),
        (any::<usize>(), 1..100u32)
            .prop_map(|(item, quantity)| Step::SetQuantity { item, quantity }),
        (any::<usize>(), any::<usize>()).prop_map(|(item, kind)| Step::MoveItem { item, kind }),
        (any::<usize>(), any::<bool>())
            .prop_map(|(item, archived)| Step::ArchiveItem { item, archived }),
        (name(), any::<bool>()).prop_map(|(name, counted)| Step::AddKind { name, counted }),
        (any::<usize>(), name()).prop_map(|(kind, name)| Step::RenameKind { kind, name }),
        (any::<usize>(), any::<bool>())
            .prop_map(|(kind, container)| Step::SetContainer { kind, container }),
        any::<usize>().prop_map(|kind| Step::ArchiveKind { kind }),
    ];
    // Often the same millisecond as the step before, so the clock's counter does the ordering.
    (step, prop_oneof![Just(0u64), 1..5u64])
}

fn patch(fields: &[(&str, Value)]) -> Patch {
    let mut patch = Patch::new();
    for (field, value) in fields {
        patch.set(*field, value.clone()).unwrap();
    }
    patch
}

/// Plays the steps on a store and returns it. Steps that point at nothing yet, or at a kind that
/// cannot take an item, are skipped, so every history is valid.
fn play(store: &Store, steps: &[(Step, u64)]) {
    let mut kinds: Vec<Id> = seed_kinds().iter().map(|kind| kind.id).collect();
    let mut open_kinds = kinds.clone();
    let mut items: Vec<Id> = Vec::new();
    let mut now = NOW;
    for (step, delta) in steps {
        now += delta;
        let pick = |ids: &[Id], index: usize| ids[index % ids.len()];
        let result = match step {
            Step::AddItem {
                name,
                kind,
                quantity,
            } => {
                let draft = ItemDraft {
                    name: name.clone(),
                    kind: pick(&kinds, *kind),
                    quantity: f64::from(*quantity),
                };
                match store.add_item(draft, now) {
                    Ok(item) => {
                        items.push(item.id);
                        Ok(())
                    }
                    Err(StoreError::UnknownKind(_)) => Ok(()),
                    Err(other) => Err(other),
                }
            }
            Step::RenameItem { item, name } if !items.is_empty() => store
                .record(
                    ITEM_ENTITY,
                    pick(&items, *item),
                    patch(&[("name", json!(name))]),
                    now,
                )
                .map(drop),
            Step::SetQuantity { item, quantity } if !items.is_empty() => store
                .record(
                    ITEM_ENTITY,
                    pick(&items, *item),
                    patch(&[("quantity", json!(quantity))]),
                    now,
                )
                .map(drop),
            Step::MoveItem { item, kind } if !items.is_empty() => store
                .record(
                    ITEM_ENTITY,
                    pick(&items, *item),
                    patch(&[("kind", json!(pick(&kinds, *kind).to_string()))]),
                    now,
                )
                .map(drop),
            Step::ArchiveItem { item, archived } if !items.is_empty() => store
                .record(
                    ITEM_ENTITY,
                    pick(&items, *item),
                    patch(&[("archived", json!(archived))]),
                    now,
                )
                .map(drop),
            Step::AddKind { name, counted } => {
                let id = Id::new(now);
                let counting = if *counted { "counted" } else { "single" };
                let created = store
                    .record(
                        KIND_ENTITY,
                        id,
                        patch(&[("name", json!(name)), ("counting", json!(counting))]),
                        now,
                    )
                    .map(drop);
                kinds.push(id);
                open_kinds.push(id);
                created
            }
            Step::RenameKind { kind, name } => store
                .record(
                    KIND_ENTITY,
                    pick(&kinds, *kind),
                    patch(&[("name", json!(name))]),
                    now,
                )
                .map(drop),
            Step::SetContainer { kind, container } => store
                .record(
                    KIND_ENTITY,
                    pick(&kinds, *kind),
                    patch(&[("container", json!(container))]),
                    now,
                )
                .map(drop),
            Step::ArchiveKind { kind } => store
                .record(
                    KIND_ENTITY,
                    pick(&kinds, *kind),
                    patch(&[("archived", json!(true))]),
                    now,
                )
                .map(drop),
            _ => Ok(()),
        };
        result.unwrap();
    }
}

/// Copies the log of `from` into the empty store `into`, in the order the keys give.
fn copy_log(from: &Store, into: &Store, keys: &[u64]) {
    type Row = (String, String, i64, i64, String, String, String);
    let reader = from.reader().unwrap();
    let mut statement = reader
        .prepare("SELECT op_id, device_id, hlc, op_version, entity, entity_id, patch FROM ops")
        .unwrap();
    let rows: Vec<Row> = statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by_key(|&i| keys[i % keys.len()]);
    let rows: Vec<Row> = order.into_iter().map(|i| rows[i].clone()).collect();
    into.with_writer(|conn| {
        for row in rows {
            conn.execute(
                "INSERT INTO ops (op_id, device_id, hlc, op_version, entity, entity_id, patch) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                (&row.0, &row.1, row.2, row.3, &row.4, &row.5, &row.6),
            )?;
        }
        Ok(())
    })
    .unwrap();
}

proptest! {
    #![proptest_config(config(48))]

    #[test]
    fn replaying_any_history_gives_the_live_tables(
        steps in prop::collection::vec(step(), 1..30),
        keys in prop::collection::vec(any::<u64>(), 1..40),
    ) {
        let live_folder = data_folder();
        let live = Store::open(live_folder.path(), NOW).unwrap();
        play(&live, &steps);
        let expected = dump(&live.reader().unwrap());

        // In place: the projection is emptied and rebuilt from the log.
        live.replay().unwrap();
        prop_assert_eq!(dump(&live.reader().unwrap()), expected.clone());

        // Into an empty database, with the log copied in an arbitrary order.
        let empty_folder = data_folder();
        let empty = Store::open(empty_folder.path(), NOW).unwrap();
        copy_log(&live, &empty, &keys);
        empty.replay().unwrap();
        prop_assert_eq!(dump(&empty.reader().unwrap()), expected);
        prop_assert_eq!(empty.list_items().unwrap(), live.list_items().unwrap());
        prop_assert_eq!(empty.list_kinds().unwrap(), live.list_kinds().unwrap());
    }
}
