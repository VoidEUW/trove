//! Items (ADR 5): the things a household owns. This is the first slice: a name, a kind and a
//! quantity. Location, condition, owners and tags arrive with later issues.

use serde_json::json;

use crate::id::Id;
use crate::kind::Kind;
use crate::op::Patch;

/// The entity type of an item in the operation log.
pub const ITEM_ENTITY: &str = "item";

/// The most of one thing an item can stand for. A sanity limit, not a rule of the household.
pub const MAX_QUANTITY: u32 = 1_000_000;

/// Why an item cannot be saved. The UI shows text for each case in the user's language.
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum ItemError {
    #[error("an item needs a name")]
    NameRequired,
    #[error("quantity must be a whole number from 1 to {MAX_QUANTITY}, got {0}")]
    QuantityInvalid(f64),
}

/// What the user typed into the Add item form. The quantity is a number as the UI sends it, so a
/// fraction, zero or a negative number reaches validation and is refused here, in one place.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemDraft {
    pub name: String,
    pub kind: Id,
    pub quantity: f64,
}

/// A draft that passed validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewItem {
    pub name: String,
    pub kind: Id,
    pub quantity: u32,
}

impl ItemDraft {
    /// The name is trimmed and must not be empty; the quantity is a whole number from 1.
    pub fn validate(self) -> Result<NewItem, ItemError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(ItemError::NameRequired);
        }
        let quantity = self.quantity;
        let whole = quantity.is_finite() && quantity.fract() == 0.0;
        if !whole || quantity < 1.0 || quantity > f64::from(MAX_QUANTITY) {
            return Err(ItemError::QuantityInvalid(quantity));
        }
        Ok(NewItem {
            name: name.to_owned(),
            kind: self.kind,
            quantity: quantity as u32,
        })
    }
}

impl NewItem {
    /// The patch of the operation that creates this item.
    pub fn to_patch(&self) -> Patch {
        let mut patch = Patch::new();
        for (field, value) in [
            ("name", json!(self.name)),
            ("kind", json!(self.kind.to_string())),
            ("quantity", json!(self.quantity)),
        ] {
            // The values are never `null` and the names are never empty.
            patch.set(field, value).ok();
        }
        patch
    }
}

/// A saved item with its kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: Id,
    pub name: String,
    pub kind: Kind,
    pub quantity: u32,
    /// When it was created, in Unix milliseconds.
    pub created_ms: u64,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn draft(name: &str, quantity: f64) -> ItemDraft {
        ItemDraft {
            name: name.to_owned(),
            kind: Id::seed(1),
            quantity,
        }
    }

    #[test]
    fn a_valid_draft_has_a_trimmed_name() {
        let item = draft("  Floor lamp \n", 2.0).validate().unwrap();
        assert_eq!(
            item,
            NewItem {
                name: "Floor lamp".to_owned(),
                kind: Id::seed(1),
                quantity: 2
            }
        );
    }

    #[test]
    fn a_blank_name_is_refused() {
        for name in ["", "   ", "\t\n"] {
            assert_eq!(
                draft(name, 1.0).validate(),
                Err(ItemError::NameRequired),
                "{name:?}"
            );
        }
    }

    #[test]
    fn quantity_is_a_whole_number_from_one() {
        for quantity in [1.0, 2.0, 99.0, f64::from(MAX_QUANTITY)] {
            assert!(draft("x", quantity).validate().is_ok(), "{quantity}");
        }
        for quantity in [
            0.0,
            -1.0,
            -0.0,
            1.5,
            0.5,
            f64::NAN,
            f64::INFINITY,
            f64::from(MAX_QUANTITY) + 1.0,
        ] {
            assert!(
                matches!(
                    draft("x", quantity).validate(),
                    Err(ItemError::QuantityInvalid(_))
                ),
                "{quantity}"
            );
        }
    }

    #[test]
    fn the_name_is_checked_before_the_quantity() {
        assert_eq!(draft("", 0.0).validate(), Err(ItemError::NameRequired));
    }

    #[test]
    fn the_creating_patch_holds_name_kind_and_quantity() {
        let item = draft("Lamp", 3.0).validate().unwrap();
        let json: Value = serde_json::to_value(item.to_patch()).unwrap();
        assert_eq!(
            json,
            json!({ "name": "Lamp", "kind": Id::seed(1).to_string(), "quantity": 3 })
        );
    }
}
