//! How many of one unit of time another holds.

/// Nanoseconds in a nanosecond, for the serde module that counts them beside the others.
#[cfg(feature = "serde")]
pub(crate) const NANOS_PER_NANOSECOND: i64 = 1;
/// Nanoseconds in a microsecond.
pub(crate) const NANOS_PER_MICROSECOND: i64 = 1_000;
/// Nanoseconds in a millisecond.
pub(crate) const NANOS_PER_MILLISECOND: i64 = 1_000_000;
/// Nanoseconds in a second.
pub(crate) const NANOS_PER_SECOND: i64 = 1_000_000_000;
/// Nanoseconds in a minute.
pub(crate) const NANOS_PER_MINUTE: i64 = SECONDS_PER_MINUTE * NANOS_PER_SECOND;
/// Nanoseconds in an hour.
pub(crate) const NANOS_PER_HOUR: i64 = SECONDS_PER_HOUR * NANOS_PER_SECOND;
/// Nanoseconds in a day of 24 hours.
pub(crate) const NANOS_PER_DAY: i64 = SECONDS_PER_DAY * NANOS_PER_SECOND;

/// Seconds in a minute.
pub(crate) const SECONDS_PER_MINUTE: i64 = 60;
/// Seconds in an hour.
pub(crate) const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
/// Seconds in a day of 24 hours.
pub(crate) const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;
