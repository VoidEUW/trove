//! The operation: one field-level change to one entity (ADR 9, ADR 16). Operations are the source of
//! truth; this type is a long-lived contract, so a change to its JSON needs a new `op_version`, an
//! upcaster in the store and a golden fixture.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::hlc::Hlc;
use crate::id::Id;

/// The version of the operation format written today.
pub const OP_VERSION: u32 = 1;

/// A field-level change: field name to its new value. A cleared field is `null` in JSON, so a field
/// can never hold a literal `null`. Fields are kept in name order, which keeps the JSON stable.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Patch(BTreeMap<String, Option<Value>>);

/// Why a patch could not be built.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PatchError {
    #[error("field `{0}` cannot be set to null; clear it instead")]
    NullValue(String),
    #[error("field name is empty")]
    EmptyField,
}

impl Patch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `field` to `value`. `null` is refused because it means "clear".
    pub fn set(&mut self, field: impl Into<String>, value: Value) -> Result<&mut Self, PatchError> {
        let field = Self::check_name(field.into())?;
        if value.is_null() {
            return Err(PatchError::NullValue(field));
        }
        self.0.insert(field, Some(value));
        Ok(self)
    }

    /// Clears `field`.
    pub fn clear(&mut self, field: impl Into<String>) -> Result<&mut Self, PatchError> {
        let field = Self::check_name(field.into())?;
        self.0.insert(field, None);
        Ok(self)
    }

    /// Each changed field with its new value, or `None` when the field is cleared.
    pub fn iter(&self) -> impl Iterator<Item = (&str, Option<&Value>)> {
        self.0
            .iter()
            .map(|(field, value)| (field.as_str(), value.as_ref()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn check_name(field: String) -> Result<String, PatchError> {
        if field.is_empty() {
            return Err(PatchError::EmptyField);
        }
        Ok(field)
    }
}

/// One change to one entity. The first operation for an `entity_id` creates the entity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub op_id: Id,
    pub device_id: Id,
    pub hlc: Hlc,
    pub op_version: u32,
    /// The entity type, such as `"item"`; the store applies the patch through a handler per type.
    pub entity: String,
    pub entity_id: Id,
    pub patch: Patch,
}

impl Operation {
    /// A new operation in the current format. Its id is made from the timestamp's time.
    pub fn new(
        device_id: Id,
        hlc: Hlc,
        entity: impl Into<String>,
        entity_id: Id,
        patch: Patch,
    ) -> Self {
        Self {
            op_id: Id::new(hlc.wall_ms()),
            device_id,
            hlc,
            op_version: OP_VERSION,
            entity: entity.into(),
            entity_id,
            patch,
        }
    }

    /// Operations are applied in this order: oldest timestamp first, the device id breaking ties.
    pub fn order_key(&self) -> (Hlc, Id) {
        (self.hlc, self.device_id)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const GOLDEN_V1: &str = include_str!("../tests/fixtures/op_v1.json");
    const GOLDEN_V1_KIND: &str = include_str!("../tests/fixtures/op_v1_kind.json");

    fn id(text: &str) -> Id {
        text.parse().unwrap()
    }

    fn golden_operation() -> Operation {
        let mut patch = Patch::new();
        patch
            .set("name", json!("Cordless drill"))
            .unwrap()
            .set("quantity", json!(2))
            .unwrap()
            .set("location", json!({ "in_room": { "room": "0199d9e0-0000-7000-8000-000000000003", "x": 1200, "y": 340 } }))
            .unwrap()
            .clear("note")
            .unwrap();
        Operation {
            op_id: id("0199d9e0-1111-7000-8000-000000000001"),
            device_id: id("0199d9e0-2222-7000-8000-000000000002"),
            hlc: Hlc::new(1_760_000_000_000, 7).unwrap(),
            op_version: 1,
            entity: "item".to_owned(),
            entity_id: id("0199d9e0-3333-7000-8000-000000000004"),
            patch,
        }
    }

    #[test]
    fn op_version_1_json_is_pinned() {
        let expected = golden_operation();
        assert_eq!(
            serde_json::from_str::<Operation>(GOLDEN_V1).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::to_string(&expected).unwrap(),
            GOLDEN_V1.trim_end()
        );
    }

    #[test]
    fn op_version_1_kind_json_is_pinned() {
        let mut patch = Patch::new();
        patch
            .set("name", json!("Pet supplies"))
            .unwrap()
            .set("counting", json!("counted"))
            .unwrap()
            .set("container", json!(true))
            .unwrap()
            .clear("archived")
            .unwrap();
        let expected = Operation {
            op_id: id("0199d9e0-1111-7000-8000-000000000011"),
            device_id: id("0199d9e0-2222-7000-8000-000000000002"),
            hlc: Hlc::new(1_760_000_000_000, 8).unwrap(),
            op_version: 1,
            entity: "kind".to_owned(),
            entity_id: id("0199d9e0-4444-7000-8000-000000000005"),
            patch,
        };
        assert_eq!(
            serde_json::from_str::<Operation>(GOLDEN_V1_KIND).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::to_string(&expected).unwrap(),
            GOLDEN_V1_KIND.trim_end()
        );
    }

    #[test]
    fn golden_fixture_writes_the_clock_as_a_string_and_a_cleared_field_as_null() {
        let json: Value = serde_json::from_str(GOLDEN_V1).unwrap();
        assert_eq!(
            json["hlc"],
            json!(golden_operation().hlc.as_i64().to_string())
        );
        assert_eq!(json["patch"]["note"], Value::Null);
        assert!(json["patch"].as_object().unwrap().contains_key("note"));
    }

    #[test]
    fn round_trips_through_json() {
        let op = Operation::new(
            Id::new(1),
            Hlc::new(1_760_000_000_000, 0).unwrap(),
            "item",
            Id::new(2),
            {
                let mut patch = Patch::new();
                patch
                    .set("name", json!("Lamp"))
                    .unwrap()
                    .clear("note")
                    .unwrap();
                patch
            },
        );
        let text = serde_json::to_string(&op).unwrap();
        assert_eq!(serde_json::from_str::<Operation>(&text).unwrap(), op);
        assert_eq!(op.op_version, OP_VERSION);
        assert_eq!(op.op_id.timestamp_ms(), 1_760_000_000_000);
    }

    #[test]
    fn a_null_in_json_reads_back_as_a_clear() {
        let patch: Patch = serde_json::from_str(r#"{"a":null,"b":false}"#).unwrap();
        let fields: Vec<_> = patch.iter().collect();
        assert_eq!(fields, vec![("a", None), ("b", Some(&json!(false)))]);
    }

    #[test]
    fn a_patch_cannot_set_null_or_name_an_empty_field() {
        let mut patch = Patch::new();
        assert_eq!(
            patch.set("a", Value::Null).unwrap_err(),
            PatchError::NullValue("a".to_owned())
        );
        assert_eq!(patch.set("", json!(1)).unwrap_err(), PatchError::EmptyField);
        assert_eq!(patch.clear("").unwrap_err(), PatchError::EmptyField);
        assert!(patch.is_empty());
    }

    #[test]
    fn a_patch_with_a_field_is_not_empty() {
        let mut patch = Patch::new();
        patch.clear("note").unwrap();
        assert!(!patch.is_empty());
    }

    #[test]
    fn operations_are_ordered_by_clock_then_device() {
        let (d1, d2) = (Id::new(1), Id::new(2));
        let at = |wall, counter, device| {
            Operation::new(
                device,
                Hlc::new(wall, counter).unwrap(),
                "item",
                Id::new(3),
                Patch::new(),
            )
        };
        let mut ops = [at(10, 1, d2), at(10, 0, d2), at(10, 1, d1), at(9, 5, d2)];
        ops.sort_by_key(Operation::order_key);
        let keys: Vec<_> = ops
            .iter()
            .map(|op| (op.hlc.wall_ms(), op.hlc.counter(), op.device_id))
            .collect();
        assert_eq!(
            keys,
            vec![(9, 5, d2), (10, 0, d2), (10, 1, d1), (10, 1, d2)]
        );
    }
}
