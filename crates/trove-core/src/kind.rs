//! Kinds of things (ADR 6): data, not code. A kind says how its items are counted and whether they
//! hold other items. Typed fields and icons come with a later issue.

use crate::id::Id;

/// The entity type of a kind in the operation log.
pub const KIND_ENTITY: &str = "kind";

/// How the items of a kind are counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Counting {
    /// Quantity is always 1.
    Single,
    /// Quantity is 1 or more.
    Counted,
}

impl Counting {
    /// The word stored in the database and written in patches.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Counted => "counted",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "single" => Some(Self::Single),
            "counted" => Some(Self::Counted),
            _ => None,
        }
    }
}

/// What a kind is called. A kind that ships with Trove stores a message key, so the UI can translate
/// it (ADR 15); a name the user typed is stored as typed and never translated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KindLabel {
    Key(String),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind {
    pub id: Id,
    pub label: KindLabel,
    pub counting: Counting,
    /// Items of this kind can hold other items.
    pub container: bool,
}

/// The kinds every installation ships with, at fixed ids (ADR 6, ADR 9). They are inserted by a
/// migration, and the store checks that its SQL and this list agree.
pub fn seed_kinds() -> Vec<Kind> {
    const SEEDS: [(&str, Counting, bool); 11] = [
        ("kind.electrical", Counting::Single, false),
        ("kind.furniture", Counting::Single, false),
        ("kind.artwork", Counting::Single, false),
        ("kind.documents", Counting::Counted, false),
        ("kind.memories", Counting::Counted, false),
        ("kind.valuables", Counting::Single, false),
        ("kind.software_key", Counting::Single, false),
        ("kind.parts_and_manuals", Counting::Counted, false),
        ("kind.tools", Counting::Single, false),
        ("kind.other", Counting::Counted, false),
        ("kind.box", Counting::Single, true),
    ];
    SEEDS
        .iter()
        .zip(1u16..)
        .map(|(&(key, counting, container), number)| Kind {
            id: Id::seed(number),
            label: KindLabel::Key(key.to_owned()),
            counting,
            container,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn eleven_seed_kinds_with_distinct_fixed_ids() {
        let kinds = seed_kinds();
        assert_eq!(kinds.len(), 11);
        let ids: HashSet<_> = kinds.iter().map(|kind| kind.id).collect();
        assert_eq!(ids.len(), 11);
        assert_eq!(seed_kinds(), kinds, "the same ids on every call");
    }

    #[test]
    fn only_the_box_is_a_container() {
        let containers: Vec<_> = seed_kinds()
            .into_iter()
            .filter(|kind| kind.container)
            .collect();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].label, KindLabel::Key("kind.box".to_owned()));
    }

    #[test]
    fn counting_round_trips_through_its_word() {
        for counting in [Counting::Single, Counting::Counted] {
            assert_eq!(Counting::parse(counting.as_str()), Some(counting));
        }
        assert_eq!(Counting::parse("many"), None);
    }
}
