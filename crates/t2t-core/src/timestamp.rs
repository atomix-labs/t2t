//! Points on the wall clock and on International Atomic Time.

use derive_more::Debug;
#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// A point on the wall clock, in nanoseconds since the Unix epoch.
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
/// assert_eq!(captured - published, Timedelta::from_micros(850), "a point minus a point");
/// assert_eq!(format!("{captured:.6}"), "2023-11-14T22:13:20.000850Z", "to the microsecond");
/// let minute = captured.floor(Timedelta::MINUTE);
/// assert_eq!(format!("{minute:.0}"), "2023-11-14T22:13:00Z", "the minute it falls in");
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct Timestamp(pub(crate) i64);

impl Timestamp {
    /// `1970-01-01T00:00:00Z`, the point the count runs from.
    pub const UNIX_EPOCH: Self = Self(0);
}

/// A point on International Atomic Time, in nanoseconds since 1970-01-01T00:00:00 TAI.
///
/// TAI never steps for a leap second, which is why PTP keeps it, and runs ahead of UTC by every
/// leap second since 1972. The offset between them is the one the kernel holds, which a time
/// service sets, so a `TaiTimestamp` never mixes with a [`Timestamp`]. It is written as RFC 3339's
/// date and time in the zone `TAI`, and parses back from it.
///
/// # Examples
/// ```
/// use t2t_core::{TaiTimestamp, Timedelta};
///
/// let synced: TaiTimestamp = "2026-09-16T07:46:12.5 TAI".parse()?;
/// let next = synced + Timedelta::from_millis(500);
///
/// assert_eq!(next.to_string(), "2026-09-16T07:46:13.000000000 TAI", "half a second on");
/// # Ok::<(), t2t_core::ParseTaiTimestampError>(())
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct TaiTimestamp(pub(crate) i64);
