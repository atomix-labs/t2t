//! Instants on the wall clock and on International Atomic Time.

/// An instant on the wall clock, in nanoseconds since the Unix epoch.
///
/// It names a moment another machine can name too, which is what makes it the stamp to capture
/// with; the clock behind it may step, so a span that must never run backwards is measured on an
/// [`Uptime`](crate::Uptime) instead. It holds 1677-09-21 to 2262-04-11, and its operators saturate
/// at either end. It is written as RFC 3339 in UTC, and parses back from it.
///
/// # Examples
/// ```
/// use t2t_core::{Timedelta, Timestamp};
///
/// let published = Timestamp::from_secs(1_700_000_000);
/// let captured = published + Timedelta::from_micros(850);
///
/// assert_eq!(captured - published, Timedelta::from_micros(850), "an instant minus an instant");
/// assert_eq!(format!("{captured:.6}"), "2023-11-14T22:13:20.000850Z", "to the microsecond");
/// let minute = captured.floor(Timedelta::MINUTE);
/// assert_eq!(format!("{minute:.0}"), "2023-11-14T22:13:00Z", "the minute it falls in");
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct Timestamp(pub(crate) i64);

const _: () = assert!(size_of::<Timestamp>() == size_of::<i64>(), "an `i64`, and nothing else");

impl Timestamp {
    /// `1970-01-01T00:00:00Z`, the instant the count runs from.
    pub const UNIX_EPOCH: Self = Self(0);
}

/// An instant on International Atomic Time, in nanoseconds since 1970-01-01T00:00:00 TAI.
///
/// TAI never steps for a leap second, which is why PTP keeps it, and runs ahead of UTC by every
/// leap second since 1972. The offset between them is the one the kernel holds, which a time
/// service sets, so a `TaiTimestamp` never mixes with a [`Timestamp`]. It is written as RFC 3339's
/// date and time with the zone `TAI`: `2026-09-16T07:46:12.123456789 TAI`.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct TaiTimestamp(pub(crate) i64);

const _: () = assert!(size_of::<TaiTimestamp>() == size_of::<i64>(), "an `i64`, and nothing else");
