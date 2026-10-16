//! Rules that must hold for any input (ADR 17, tier 1): the clock, patches, operations and item
//! validation. Cases are bounded; inputs proptest finds are kept in `properties.proptest-regressions`.

use std::collections::BTreeMap;

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};
use trove_core::{
    Clock, ClockError, Hlc, Id, ItemDraft, ItemError, MAX_DRIFT_MS, MAX_QUANTITY, Operation, Patch,
    PatchError,
};

const MAX_WALL_MS: u64 = (1 << 47) - 1;

/// Inputs that failed once are kept next to this file and replayed first.
fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/properties.proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}
/// Wall times below 2^40 ms (the year 2004 to 36812) leave room to add hours and never hit the 47-bit limit.
const WALL: std::ops::Range<u64> = 0..(1 << 40);

fn hlc() -> impl Strategy<Value = Hlc> {
    (0..=MAX_WALL_MS, any::<u16>()).prop_map(|(wall, counter)| Hlc::new(wall, counter).unwrap())
}

fn value() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<i64>().prop_map(|n| json!(n)),
        any::<bool>().prop_map(|b| json!(b)),
        ".{0,12}".prop_map(|s| json!(s)),
    ]
}

fn patch_steps() -> impl Strategy<Value = Vec<(String, Option<Value>)>> {
    prop::collection::vec(("[a-z]{1,4}", prop::option::of(value())), 0..12)
}

fn patch_from(steps: &[(String, Option<Value>)]) -> Patch {
    let mut patch = Patch::new();
    for (field, value) in steps {
        match value {
            Some(value) => patch.set(field.clone(), value.clone()).unwrap(),
            None => patch.clear(field.clone()).unwrap(),
        };
    }
    patch
}

fn operation() -> impl Strategy<Value = Operation> {
    (WALL, WALL, WALL, hlc(), "[a-z]{1,6}", patch_steps()).prop_map(
        |(op_ms, device_ms, entity_ms, hlc, entity, steps)| Operation {
            op_id: Id::new(op_ms),
            device_id: Id::new(device_ms),
            hlc,
            op_version: 1,
            entity,
            entity_id: Id::new(entity_ms),
            patch: patch_from(&steps),
        },
    )
}

proptest! {
    #![proptest_config(config(128))]

    #[test]
    fn a_clock_value_keeps_its_parts_and_every_form_round_trips(wall in 0..=MAX_WALL_MS, counter in any::<u16>()) {
        let hlc = Hlc::new(wall, counter).unwrap();
        prop_assert_eq!((hlc.wall_ms(), hlc.counter()), (wall, counter));
        prop_assert!(hlc.as_i64() >= 0);
        prop_assert_eq!(Hlc::from_i64(hlc.as_i64()).unwrap(), hlc);
        prop_assert_eq!(hlc.to_string().parse::<Hlc>().unwrap(), hlc);
        let json = serde_json::to_string(&hlc).unwrap();
        prop_assert_eq!(serde_json::from_str::<Hlc>(&json).unwrap(), hlc);
    }

    #[test]
    fn clock_values_order_by_time_then_counter(
        a in (0..=MAX_WALL_MS, any::<u16>()),
        b in (0..=MAX_WALL_MS, any::<u16>()),
    ) {
        let (x, y) = (Hlc::new(a.0, a.1).unwrap(), Hlc::new(b.0, b.1).unwrap());
        prop_assert_eq!(x.cmp(&y), a.cmp(&b));
        prop_assert_eq!(x.as_i64().cmp(&y.as_i64()), a.cmp(&b));
    }

    #[test]
    fn a_time_beyond_47_bits_is_refused(wall in (MAX_WALL_MS + 1)..=u64::MAX, counter in any::<u16>()) {
        prop_assert_eq!(Hlc::new(wall, counter), Err(ClockError::WallOutOfRange(wall)));
    }

    #[test]
    fn only_non_negative_numbers_are_clock_values(value in any::<i64>()) {
        prop_assert_eq!(Hlc::from_i64(value).is_ok(), value >= 0);
    }

    #[test]
    fn tick_rises_for_any_sequence_of_wall_clock_readings(readings in prop::collection::vec(WALL, 1..40)) {
        let mut clock = Clock::default();
        let mut previous = Hlc::ZERO;
        let mut latest = 0;
        for now in readings {
            let next = clock.tick(now).unwrap();
            latest = latest.max(now);
            prop_assert!(next > previous);
            prop_assert_eq!(next.wall_ms(), latest, "time never goes back, even if the wall clock does");
            prop_assert_eq!(clock.last(), next);
            previous = next;
        }
    }

    #[test]
    fn receive_follows_both_clocks_or_fails_and_changes_nothing(
        now in 1_000_000_000_000u64..2_000_000_000_000,
        local_offset in -7_200_000i64..7_200_000,
        remote_offset in -7_200_000i64..7_200_000,
        local_counter in any::<u16>(),
        remote_counter in any::<u16>(),
    ) {
        let at = |offset: i64, counter| Hlc::new(now.saturating_add_signed(offset), counter).unwrap();
        let (last, remote) = (at(local_offset, local_counter), at(remote_offset, remote_counter));
        let mut clock = Clock::new(last);
        match clock.receive(remote, now) {
            Ok(next) => {
                prop_assert!(remote.wall_ms().saturating_sub(now) <= MAX_DRIFT_MS);
                prop_assert!(next > last && next > remote);
                prop_assert_eq!(next.wall_ms(), now.max(last.wall_ms()).max(remote.wall_ms()));
                prop_assert_eq!(clock.last(), next);
            }
            Err(error) => {
                let ahead_ms = remote.wall_ms().saturating_sub(now);
                match error {
                    ClockError::DriftExceeded { ahead_ms: reported } => {
                        prop_assert!(ahead_ms > MAX_DRIFT_MS);
                        prop_assert_eq!(reported, ahead_ms);
                    }
                    ClockError::CounterOverflow => prop_assert!(ahead_ms <= MAX_DRIFT_MS),
                    other => prop_assert!(false, "unexpected {other:?}"),
                }
                prop_assert_eq!(clock.last(), last);
            }
        }
    }

    #[test]
    fn a_patch_holds_the_last_write_to_each_field(steps in patch_steps()) {
        let mut model: BTreeMap<&str, Option<&Value>> = BTreeMap::new();
        for (field, value) in &steps {
            model.insert(field, value.as_ref());
        }
        let patch = patch_from(&steps);
        prop_assert_eq!(patch.iter().collect::<Vec<_>>(), model.into_iter().collect::<Vec<_>>());
        prop_assert_eq!(patch.is_empty(), steps.is_empty());

        let text = serde_json::to_string(&patch).unwrap();
        prop_assert_eq!(serde_json::from_str::<Patch>(&text).unwrap(), patch);
    }

    #[test]
    fn a_patch_never_holds_a_literal_null_or_an_unnamed_field(field in "[a-z]{1,4}") {
        let mut patch = Patch::new();
        prop_assert_eq!(patch.set(field.clone(), Value::Null).unwrap_err(), PatchError::NullValue(field));
        prop_assert_eq!(patch.set("", json!(1)).unwrap_err(), PatchError::EmptyField);
        prop_assert_eq!(patch.clear("").unwrap_err(), PatchError::EmptyField);
        prop_assert!(patch.is_empty());
    }

    #[test]
    fn an_operation_round_trips_through_json(op in operation()) {
        let text = serde_json::to_string(&op).unwrap();
        prop_assert_eq!(serde_json::from_str::<Operation>(&text).unwrap(), op);
    }

    #[test]
    fn operations_sort_by_clock_then_device_whatever_order_they_arrive_in(
        (ops, shuffled) in prop::collection::vec(operation(), 0..16)
            .prop_flat_map(|ops| (Just(ops.clone()), Just(ops).prop_shuffle())),
    ) {
        let keys = |mut ops: Vec<Operation>| {
            ops.sort_by_key(Operation::order_key);
            ops.iter().map(Operation::order_key).collect::<Vec<_>>()
        };
        let sorted = keys(ops);
        prop_assert!(sorted.windows(2).all(|pair| pair[0] <= pair[1]));
        prop_assert_eq!(keys(shuffled), sorted);
    }

    #[test]
    fn ids_made_later_sort_later(a in WALL, b in WALL) {
        prop_assume!(a != b);
        prop_assert_eq!(Id::new(a) < Id::new(b), a < b);
        prop_assert_eq!(Id::new(a).timestamp_ms(), a);
    }
}

fn quantity() -> impl Strategy<Value = f64> {
    prop_oneof![
        any::<f64>(),
        (0..=MAX_QUANTITY + 2).prop_map(f64::from),
        (0..=2 * MAX_QUANTITY).prop_map(|n| f64::from(n) / 2.0),
        Just(f64::NAN),
        Just(f64::INFINITY),
        Just(-0.0),
    ]
}

/// One character of a name, and whether a person can see it. Whitespace and the zero-width and other
/// invisible characters are not visible; letters, marks and directional marks are.
fn name_char() -> impl Strategy<Value = (char, bool)> {
    prop_oneof![
        prop::sample::select(vec![
            ' ', '\t', '\n', '\r', '\u{a0}', '\u{2003}', '\u{3000}'
        ])
        .prop_map(|c| (c, false)),
        prop::sample::select(vec![
            '\u{ad}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}'
        ])
        .prop_map(|c| (c, false)),
        prop::sample::select(vec!['\u{0}', '\u{7}', '\u{7f}', '\u{85}']).prop_map(|c| (c, false)),
        prop::sample::select(vec![
            'a',
            'Z',
            '7',
            '\u{e9}',
            '\u{6f22}',
            '\u{1f389}',
            '\u{200e}',
            '-'
        ])
        .prop_map(|c| (c, true)),
    ]
}

fn draft(name: String, quantity: f64) -> ItemDraft {
    ItemDraft {
        name,
        kind: Id::seed(1),
        quantity,
    }
}

proptest! {
    #![proptest_config(config(256))]

    #[test]
    fn a_quantity_is_accepted_exactly_when_it_is_a_whole_number_from_one_to_the_maximum(q in quantity()) {
        let expected = q.is_finite() && q.fract() == 0.0 && q >= 1.0 && q <= f64::from(MAX_QUANTITY);
        let result = draft("Lamp".to_owned(), q).validate();
        match result {
            Ok(item) => {
                prop_assert!(expected, "{q} was accepted");
                prop_assert_eq!(f64::from(item.quantity), q);
            }
            Err(error) => {
                prop_assert!(!expected, "{q} was refused");
                prop_assert!(matches!(error, ItemError::QuantityInvalid(_)));
            }
        }
    }

    #[test]
    fn a_name_is_accepted_exactly_when_it_has_a_visible_character_and_is_stored_trimmed(
        chars in prop::collection::vec(name_char(), 0..12),
    ) {
        let name: String = chars.iter().map(|(c, _)| c).collect();
        let has_visible = chars.iter().any(|(_, visible)| *visible);
        let result = draft(name.clone(), 1.0).validate();
        if has_visible {
            let item = result.unwrap();
            prop_assert_eq!(&item.name, name.trim());
            // Validating what validation returned changes nothing.
            prop_assert_eq!(draft(item.name.clone(), 1.0).validate().unwrap(), item);
        } else {
            prop_assert_eq!(result, Err(ItemError::NameRequired));
        }
    }
}
