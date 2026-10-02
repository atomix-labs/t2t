//! Instants on the clocks that count from boot.

use core::fmt;

use crate::Timedelta;

/// An instant on the monotonic clock, in nanoseconds since an origin near boot.
///
/// It never steps, and every process on a machine reads the same clock, so a deadline is an
/// [`Uptime`] plus a [`Timedelta`]; it names no moment off this machine, which a
/// [`Timestamp`](crate::Timestamp) does. It is written as the span since its origin.
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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct Uptime(pub(crate) i64);

/// An instant on the monotonic clock at the hardware's own rate, in nanoseconds since an origin
/// near boot.
///
/// No time service adjusts its rate, so it drifts from an [`Uptime`] by as much as one slews the
/// monotonic clock, and the two never mix. It is written as the span since its origin.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct RawUptime(pub(crate) i64);

/// An instant on the boot clock, in nanoseconds since boot, counting the time the machine was
/// suspended.
///
/// It is the monotonic clock plus every suspension, so it never mixes with an [`Uptime`]. It is
/// written as the span since boot.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct BootTime(pub(crate) i64);

/// A point counted from boot: its layout, and its spelling as the span since its origin.
macro_rules! since_boot {
    ($point:ident) => {
        const _: () =
            assert!(size_of::<$point>() == size_of::<i64>(), "an `i64`, and nothing else");

        /// The span since the origin, as [`Timedelta`] writes it.
        impl fmt::Display for $point {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&Timedelta(self.0), formatter)
            }
        }

        /// As [`Display`](fmt::Display) writes it.
        impl fmt::Debug for $point {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(self, formatter)
            }
        }
    };
}

since_boot!(Uptime);
since_boot!(RawUptime);
since_boot!(BootTime);
