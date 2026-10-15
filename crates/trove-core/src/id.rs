//! Ids are UUIDv7 and generated here, never in the store or the UI.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// A UUIDv7. Used for operations, devices and every entity. Sorting follows the byte order, which
/// equals the order of the hyphenated lowercase text form SQLite stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(Uuid);

impl Id {
    /// A new id whose timestamp part is `now_ms` (Unix milliseconds). The caller supplies the time
    /// because this crate does no IO; the random part comes from the operating system.
    pub fn new(now_ms: u64) -> Self {
        let timestamp =
            Timestamp::from_unix(NoContext, now_ms / 1000, (now_ms % 1000) as u32 * 1_000_000);
        Self(Uuid::new_v7(timestamp))
    }

    /// A fixed id for a row every installation ships with, such as a seed kind (ADR 9). `n` tells the
    /// rows apart. The id is a valid version 7 UUID whose timestamp is zero, so it sorts before every
    /// id made by [`Id::new`].
    pub const fn seed(n: u16) -> Self {
        const SEED_TIMESTAMP: u128 = 0;
        const VERSION_7: u128 = 0x7 << 76;
        const VARIANT: u128 = 0b10 << 62;
        Self(Uuid::from_u128(
            (SEED_TIMESTAMP << 80) | VERSION_7 | VARIANT | n as u128,
        ))
    }

    /// The milliseconds since the Unix epoch encoded in the id.
    pub fn timestamp_ms(self) -> u64 {
        self.0
            .get_timestamp()
            .map(|ts| {
                let (secs, nanos) = ts.to_unix();
                secs * 1000 + u64::from(nanos / 1_000_000)
            })
            .unwrap_or(0)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(f)
    }
}

/// Why a text is not an [`Id`].
#[derive(Debug, thiserror::Error)]
pub enum IdError {
    #[error("not a UUID: {0}")]
    Malformed(#[from] uuid::Error),
    #[error("not a version 7 UUID: {0}")]
    NotV7(Uuid),
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        if uuid.get_version_num() != 7 {
            return Err(IdError::NotV7(uuid));
        }
        Ok(Self(uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ids_are_version_7_and_carry_the_time() {
        let id = Id::new(1_760_000_000_123);
        assert_eq!(id.0.get_version_num(), 7);
        assert_eq!(id.timestamp_ms(), 1_760_000_000_123);
    }

    #[test]
    fn seed_ids_are_fixed_and_differ_by_number() {
        assert_eq!(Id::seed(1), Id::seed(1));
        assert_ne!(Id::seed(1), Id::seed(2));
    }

    #[test]
    fn seed_ids_are_version_7_ids() {
        assert_eq!(Id::seed(7).0.get_version_num(), 7);
        assert_eq!(Id::seed(7).to_string().parse::<Id>().unwrap(), Id::seed(7));
    }

    #[test]
    fn seed_ids_sort_before_every_id_made_by_new() {
        assert!(Id::seed(u16::MAX) < Id::new(1));
    }

    #[test]
    fn a_seed_id_has_a_stable_text_form() {
        assert_eq!(
            Id::seed(1).to_string(),
            "00000000-0000-7000-8000-000000000001"
        );
    }

    #[test]
    fn ids_made_later_sort_later() {
        assert!(Id::new(1_000) < Id::new(2_000));
    }

    #[test]
    fn text_form_round_trips() {
        let id = Id::new(1_760_000_000_000);
        assert_eq!(id.to_string().parse::<Id>().unwrap(), id);
    }

    #[test]
    fn other_uuid_versions_are_refused() {
        let v4 = "67e55044-10b1-426f-9247-bb680e5fe0c8";
        assert!(matches!(v4.parse::<Id>(), Err(IdError::NotV7(_))));
        assert!(matches!("nope".parse::<Id>(), Err(IdError::Malformed(_))));
    }
}
