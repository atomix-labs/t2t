//! Points on the clocks that count from boot.

use core::str::FromStr;

use derive_more::{Debug, Display};
#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{ParseTimedeltaError, Timedelta};

/// A point on the monotonic clock, in nanoseconds since an origin near boot.
///
/// It never steps, and every process on a machine reads the same clock, so a deadline is an
/// [`Uptime`] plus a [`Timedelta`]; it names no moment off this machine, which a
/// [`Timestamp`](crate::Timestamp) does. It is written as the span since its origin, and parses
/// back from it.
///
/// # Examples
/// ```
/// use t2t_core::{Timedelta, Uptime};
///
/// let start = Uptime::from_secs(42);
/// let deadline = start + Timedelta::from_millis(50);
///
/// assert!(start < deadline, "a deadline lies ahead");
/// assert_eq!(deadline.to_string(), "42s50ms", "written as the span since the origin");
/// ```
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[display("{}", Timedelta(*_0))]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct Uptime(pub(crate) i64);

/// A point on the monotonic clock at the hardware's own rate, in nanoseconds since an origin
/// near boot.
///
/// No time service adjusts its rate, so it drifts from an [`Uptime`] by as much as one slews the
/// monotonic clock, and the two never mix. It is written as the span since its origin, and parses
/// back from it.
///
/// # Examples
/// ```
/// use t2t_core::{RawUptime, Timedelta};
///
/// let (start, end) = (RawUptime::from_secs(42), RawUptime::from_millis(42_010));
/// assert_eq!(end - start, Timedelta::from_millis(10), "a span free of any slewing");
/// ```
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[display("{}", Timedelta(*_0))]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct RawUptime(pub(crate) i64);

/// A point on the boot clock, in nanoseconds since boot, the machine's suspensions counted.
///
/// It is the monotonic clock plus every suspension, so it never mixes with an [`Uptime`]. It is
/// written as the span since boot, and parses back from it.
///
/// # Examples
/// ```
/// use t2t_core::{BootUptime, Timedelta};
///
/// let lease = BootUptime::from_secs(3_600) + Timedelta::HOUR;
/// assert_eq!(lease.to_string(), "2h", "an expiry a suspension cannot carry past");
/// ```
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[display("{}", Timedelta(*_0))]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct BootUptime(pub(crate) i64);

/// Reading a point counted from boot from the span since its origin.
macro_rules! read_as_span {
    ($point:ident) => {
        /// Reads what [`Display`](core::fmt::Display) writes: the span since the origin.
        impl FromStr for $point {
            type Err = ParseTimedeltaError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                text.parse().map(|span: Timedelta| Self(span.0))
            }
        }
    };
}

read_as_span!(Uptime);
read_as_span!(RawUptime);
read_as_span!(BootUptime);

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;

    use crate::{BootUptime, ParseTimedeltaError, RawUptime, Uptime};

    #[test]
    fn a_point_from_boot_reads_back_from_its_span() {
        let uptime = Uptime::from_millis(90_250);
        assert_eq!(uptime.to_string(), "1m30s250ms", "written as the span since the origin");
        assert_eq!("1m30s250ms".parse(), Ok(uptime), "and read back");
        assert_eq!("2h".parse(), Ok(BootUptime::from_secs(7_200)), "on every clock from boot");
        assert_eq!("-".parse::<RawUptime>(), Err(ParseTimedeltaError), "refused as a span is");
    }

    #[test]
    fn a_width_pads_a_point_from_boot_as_it_pads_a_span() {
        assert_eq!(format!("[{:>8}]", Uptime::from_millis(500)), "[   500ms]", "padded");
        assert_eq!(format!("{:?}", RawUptime::from_secs(2)), "2s", "and debugged as written");
    }
}
