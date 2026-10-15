//! The hybrid logical clock (ADR 16). A timestamp packs 47 bits of Unix milliseconds and a 16-bit
//! counter into a positive `i64`, so it fits a SQLite `INTEGER` and sorts as a number. In JSON it is
//! a decimal string, because the value exceeds 2^53 and a floating-point reader would round it.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// A remote timestamp further ahead of the local wall clock than this is refused (1 hour).
pub const MAX_DRIFT_MS: u64 = 60 * 60 * 1000;

const COUNTER_BITS: u32 = 16;
const MAX_WALL_MS: u64 = (1 << 47) - 1;

/// Why a timestamp could not be made or accepted.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ClockError {
    #[error("{0} ms does not fit in 47 bits")]
    WallOutOfRange(u64),
    #[error("more than 65535 events in one millisecond")]
    CounterOverflow,
    #[error(
        "remote clock is {ahead_ms} ms ahead of this device, more than the {MAX_DRIFT_MS} ms limit"
    )]
    DriftExceeded { ahead_ms: u64 },
    #[error("not a valid timestamp: {0}")]
    Malformed(String),
}

/// A hybrid logical timestamp. Ordering is the order of the packed integer: time first, counter second.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hlc(i64);

impl Hlc {
    /// Before every real timestamp.
    pub const ZERO: Self = Self(0);

    pub fn new(wall_ms: u64, counter: u16) -> Result<Self, ClockError> {
        if wall_ms > MAX_WALL_MS {
            return Err(ClockError::WallOutOfRange(wall_ms));
        }
        // 47 + 16 = 63 bits: always a positive i64.
        Ok(Self(
            ((wall_ms << COUNTER_BITS) | u64::from(counter)) as i64,
        ))
    }

    pub fn wall_ms(self) -> u64 {
        (self.0 as u64) >> COUNTER_BITS
    }

    pub fn counter(self) -> u16 {
        (self.0 as u64 & u64::from(u16::MAX)) as u16
    }

    /// The value stored in SQLite.
    pub fn as_i64(self) -> i64 {
        self.0
    }

    /// Reads a stored value; a negative integer cannot come from [`Hlc::as_i64`].
    pub fn from_i64(value: i64) -> Result<Self, ClockError> {
        if value < 0 {
            return Err(ClockError::Malformed(value.to_string()));
        }
        Ok(Self(value))
    }
}

impl fmt::Display for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Hlc {
    type Err = ClockError;

    /// Accepts only the canonical decimal text, so `"007"` and `"+7"` are refused.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let malformed = || ClockError::Malformed(s.to_owned());
        let value: i64 = s.parse().map_err(|_| malformed())?;
        if value.to_string() != s {
            return Err(malformed());
        }
        Self::from_i64(value)
    }
}

impl Serialize for Hlc {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Hlc {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(de::Error::custom)
    }
}

/// One device's clock. It never reads the wall clock; callers pass `now_ms`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    last: Hlc,
}

impl Clock {
    /// Continues from the newest timestamp this device has seen (the highest `hlc` in `ops`).
    pub fn new(last: Hlc) -> Self {
        Self { last }
    }

    pub fn last(&self) -> Hlc {
        self.last
    }

    /// The timestamp for a local event. Always greater than every earlier one, even if the wall
    /// clock stood still or went back.
    pub fn tick(&mut self, now_ms: u64) -> Result<Hlc, ClockError> {
        let wall = now_ms.max(self.last.wall_ms());
        let counter = if wall == self.last.wall_ms() {
            self.last
                .counter()
                .checked_add(1)
                .ok_or(ClockError::CounterOverflow)?
        } else {
            0
        };
        self.advance(wall, counter)
    }

    /// Folds in a timestamp from another device and returns one greater than both. A remote
    /// timestamp more than [`MAX_DRIFT_MS`] ahead of `now_ms` is refused and the clock is unchanged.
    pub fn receive(&mut self, remote: Hlc, now_ms: u64) -> Result<Hlc, ClockError> {
        let ahead_ms = remote.wall_ms().saturating_sub(now_ms);
        if ahead_ms > MAX_DRIFT_MS {
            return Err(ClockError::DriftExceeded { ahead_ms });
        }
        let wall = now_ms.max(self.last.wall_ms()).max(remote.wall_ms());
        let seen = [self.last, remote]
            .into_iter()
            .filter(|hlc| hlc.wall_ms() == wall)
            .map(Hlc::counter)
            .max();
        let counter = match seen {
            Some(counter) => counter.checked_add(1).ok_or(ClockError::CounterOverflow)?,
            None => 0,
        };
        self.advance(wall, counter)
    }

    fn advance(&mut self, wall_ms: u64, counter: u16) -> Result<Hlc, ClockError> {
        let next = Hlc::new(wall_ms, counter)?;
        self.last = next;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_760_000_000_000;

    #[test]
    fn packs_47_bits_of_time_and_16_of_counter() {
        let hlc = Hlc::new(NOW, 7).unwrap();
        assert_eq!(hlc.wall_ms(), NOW);
        assert_eq!(hlc.counter(), 7);
        assert_eq!(hlc.as_i64(), (NOW as i64) << 16 | 7);
        assert!(hlc.as_i64() > 0);
    }

    #[test]
    fn the_largest_timestamp_is_still_a_positive_i64() {
        let hlc = Hlc::new(MAX_WALL_MS, u16::MAX).unwrap();
        assert_eq!(hlc.as_i64(), i64::MAX);
        assert_eq!(
            Hlc::new(MAX_WALL_MS + 1, 0),
            Err(ClockError::WallOutOfRange(MAX_WALL_MS + 1))
        );
    }

    #[test]
    fn the_drift_limit_is_one_hour() {
        assert_eq!(MAX_DRIFT_MS, 3_600_000);
    }

    #[test]
    fn zero_is_a_valid_clock_value_and_negative_numbers_are_not() {
        assert_eq!(Hlc::from_i64(0), Ok(Hlc::ZERO));
        assert_eq!(
            Hlc::from_i64(-1),
            Err(ClockError::Malformed("-1".to_owned()))
        );
    }

    #[test]
    fn orders_by_time_then_counter() {
        assert!(Hlc::new(NOW, 9).unwrap() < Hlc::new(NOW + 1, 0).unwrap());
        assert!(Hlc::new(NOW, 1).unwrap() < Hlc::new(NOW, 2).unwrap());
    }

    #[test]
    fn json_form_is_a_decimal_string() {
        let hlc = Hlc::new(NOW, 7).unwrap();
        let json = serde_json::to_string(&hlc).unwrap();
        assert_eq!(json, format!("\"{}\"", hlc.as_i64()));
        assert_eq!(serde_json::from_str::<Hlc>(&json).unwrap(), hlc);
    }

    #[test]
    fn json_form_refuses_numbers_and_non_canonical_text() {
        assert!(serde_json::from_str::<Hlc>("123").is_err());
        for text in ["\"007\"", "\"+7\"", "\"-1\"", "\"\"", "\"1.5\"", "\"abc\""] {
            assert!(serde_json::from_str::<Hlc>(text).is_err(), "{text}");
        }
    }

    #[test]
    fn tick_is_monotonic_when_time_stands_still_or_goes_back() {
        let mut clock = Clock::default();
        let a = clock.tick(NOW).unwrap();
        let b = clock.tick(NOW).unwrap();
        let c = clock.tick(NOW - 5_000).unwrap();
        let d = clock.tick(NOW + 1).unwrap();
        assert!(a < b && b < c && c < d);
        assert_eq!(
            (a.counter(), b.counter(), c.counter(), d.counter()),
            (0, 1, 2, 0)
        );
        assert_eq!(c.wall_ms(), NOW);
        assert_eq!(d.wall_ms(), NOW + 1);
    }

    #[test]
    fn tick_fails_instead_of_wrapping_the_counter() {
        let mut clock = Clock::new(Hlc::new(NOW, u16::MAX).unwrap());
        assert_eq!(clock.tick(NOW), Err(ClockError::CounterOverflow));
        assert_eq!(clock.last(), Hlc::new(NOW, u16::MAX).unwrap());
    }

    #[test]
    fn receive_returns_a_timestamp_after_both() {
        let mut clock = Clock::new(Hlc::new(NOW, 3).unwrap());
        let remote = Hlc::new(NOW, 8).unwrap();
        let next = clock.receive(remote, NOW).unwrap();
        assert!(next > remote && next > Hlc::new(NOW, 3).unwrap());
        assert_eq!(next, Hlc::new(NOW, 9).unwrap());

        let later = Hlc::new(NOW + 500, 0).unwrap();
        assert_eq!(
            clock.receive(later, NOW).unwrap(),
            Hlc::new(NOW + 500, 1).unwrap()
        );

        let older = Hlc::new(NOW - 10, 99).unwrap();
        assert!(clock.receive(older, NOW + 600).unwrap() > later);
    }

    #[test]
    fn receive_uses_the_wall_clock_when_it_is_ahead_of_both() {
        let mut clock = Clock::new(Hlc::new(NOW, 3).unwrap());
        let next = clock
            .receive(Hlc::new(NOW - 1, 9).unwrap(), NOW + 100)
            .unwrap();
        assert_eq!(next, Hlc::new(NOW + 100, 0).unwrap());
    }

    #[test]
    fn receive_refuses_a_clock_beyond_the_drift_limit_and_stays_unchanged() {
        let mut clock = Clock::new(Hlc::new(NOW, 3).unwrap());
        let at_limit = Hlc::new(NOW + MAX_DRIFT_MS, 0).unwrap();
        assert!(clock.receive(at_limit, NOW).is_ok());

        let mut clock = Clock::new(Hlc::new(NOW, 3).unwrap());
        let too_far = Hlc::new(NOW + MAX_DRIFT_MS + 1, 0).unwrap();
        assert_eq!(
            clock.receive(too_far, NOW),
            Err(ClockError::DriftExceeded {
                ahead_ms: MAX_DRIFT_MS + 1
            })
        );
        assert_eq!(clock.last(), Hlc::new(NOW, 3).unwrap());
    }
}
