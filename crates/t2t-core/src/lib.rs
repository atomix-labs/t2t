//! Time values: instants on the wall and monotonic clocks, counter readings, and the spans between
//! them.
//!
//! Each timeline has its own point, and each kind of count its own span. A point minus a point
//! is a span, a point plus a span is a point, and points of two timelines never mix:
//!
//! ```text
//! point          timeline                                   span
//! Timestamp      the wall clock, from the Unix epoch        Timedelta
//! TaiTimestamp   International Atomic Time, from 1970 TAI   Timedelta
//! Uptime         the monotonic clock, from near boot        Timedelta
//! RawUptime      the monotonic clock at the hardware's rate Timedelta
//! BootTime       the boot clock, counting suspensions       Timedelta
//! Tick           a hardware counter                         Ticks, worth a Timedelta at a TickRate
//! ```
//!
//! - [`Timestamp`] names a moment other machines name too; [`TaiTimestamp`] names one on the
//!   timescale PTP keeps.
//! - [`Uptime`], [`RawUptime`] and [`BootTime`] never step, and name no moment off the machine.
//! - [`Tick`] is a hardware counter's reading, one instruction to take.
//! - [`Timedelta`] and [`Ticks`] are the spans; [`TickRate`] converts one to the other.
//! - [`Timed`] is a value and the stamp it was captured with.
//! - [`UtcDateTime`] is an instant read as a date and a time of day.
//! - [`TimePoint`] is what every point shares, for code generic over them.
//! - [`ParseTimedeltaError`], [`ParseTimestampError`] and [`OutOfRangeError`] are the refusals.
//!
//! Every value is an `i64`, and its operators saturate at the ends of the range rather than
//! overflow, each with a `checked_*` twin.
//!
//! ```
//! use t2t_core::{Timed, Timedelta, Timestamp};
//!
//! let stale: Timedelta = "1s".parse()?;
//! let captured = Timestamp::from_secs(1_700_000_000);
//! let quote = Timed::new(captured, 101_u64);
//!
//! assert!(quote.elapsed(captured + Timedelta::from_millis(1_500)) > stale, "stale by now");
//! # Ok::<(), t2t_core::ParseTimedeltaError>(())
//! ```
//!
//! # Crate features
//!
//! None is on by default, and nothing reaches the operating system unless `std` is named.
//!
//! | Feature     | Adds                                                                                 |
//! | ----------- | ------------------------------------------------------------------------------------ |
//! | `std`       | `Timestamp` to and from `std::time::SystemTime`, on 64-bit Linux and macOS           |
//! | `serde`     | the string spellings, and the `serde` modules for counts in a named unit             |
//! | `schemars`  | `JsonSchema` for `Timestamp` and `Timedelta`; turns `serde` on                       |
//! | `zerocopy`  | `FromBytes`, `IntoBytes` and the rest where each type can honour them; native-endian |
//! | `chrono-04` | `Timestamp` and `Timedelta` to and from chrono 0.4's `DateTime` and `TimeDelta`      |
//! | `jiff-02`   | `Timestamp` and `Timedelta` to and from jiff 0.2's `Timestamp` and `SignedDuration`  |
//! | `time-03`   | `Timestamp` and `Timedelta` to and from time 0.3's `OffsetDateTime` and `Duration`   |

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(test, feature = "schemars"))]
extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

mod errors;
mod interop;
mod ops;
mod point;
mod rfc3339;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
))]
mod system_time;
mod text;
mod tick;
mod tick_rate;
mod timed;
mod timedelta;
mod timestamp;
mod uptime;
mod utc;

pub use crate::errors::{OutOfRangeError, ParseTimedeltaError, ParseTimestampError};
pub use crate::point::TimePoint;
pub use crate::tick::{Tick, Ticks};
pub use crate::tick_rate::TickRate;
pub use crate::timed::Timed;
pub use crate::timedelta::Timedelta;
pub use crate::timestamp::{TaiTimestamp, Timestamp};
pub use crate::uptime::{BootTime, RawUptime, Uptime};
pub use crate::utc::UtcDateTime;
