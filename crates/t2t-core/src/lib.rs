//! Time values: points on the wall and monotonic clocks, counter readings, and the spans between
//! them.
//!
//! Each timeline has its own point, and each kind of count its own span. A point minus a point
//! is a span, a point plus a span is a point, and points of two timelines never mix:
//!
//! | Point            | Timeline                                   | Span          |
//! | ---------------- | ------------------------------------------ | ------------- |
//! | [`Timestamp`]    | the wall clock, from the Unix epoch        | [`Timedelta`] |
//! | [`TaiTimestamp`] | International Atomic Time, from 1970 TAI   | [`Timedelta`] |
//! | [`Uptime`]       | the monotonic clock, from near boot        | [`Timedelta`] |
//! | [`RawUptime`]    | the monotonic clock at the hardware's rate | [`Timedelta`] |
//! | [`BootUptime`]   | the boot clock, suspensions counted        | [`Timedelta`] |
//! | [`Tickstamp`]    | a hardware counter, from its own origin    | [`Tickdelta`] |
//!
//! Every point and span is an `i64`, and its operators saturate at the ends of the range rather
//! than overflow, each with a `checked_*` twin. Every value has a spelling, which `Display` writes
//! and `FromStr` reads back.
//!
//! # Types
//!
//! - **Points.** [`Timestamp`] and [`TaiTimestamp`] name a moment another machine names too;
//!   [`Uptime`], [`RawUptime`] and [`BootUptime`] never step; a [`Tickstamp`] is a counter's
//!   reading. [`TimePoint`] is what they share, for code generic over them.
//! - **Spans.** [`Timedelta`] counts nanoseconds and [`Tickdelta`] a counter's ticks; a
//!   [`TickRate`] turns one into the other.
//! - **Views.** [`Timed`] is a value and the stamp it was captured with; [`UtcDateTime`] is a
//!   timestamp read as a date and a time of day.
//! - **Refusals.** [`ParseTimestampError`], [`ParseTaiTimestampError`], [`ParseTimedeltaError`],
//!   [`ParseTickdeltaError`] and [`ParseTickRateError`] for a spelling that does not read;
//!   [`OutOfRangeError`] for a time another type cannot hold.
//!
//! # Examples
//! ```
//! use t2t_core::{Timed, Timedelta, Timestamp};
//!
//! let stale: Timedelta = "1s".parse()?;
//! let captured = Timestamp::from_secs(1_700_000_000);
//! let sample = Timed::new(captured, 101_u64);
//!
//! assert!(sample.elapsed(captured + Timedelta::from_millis(1_500)) > stale, "stale by now");
//! # Ok::<(), t2t_core::ParseTimedeltaError>(())
//! ```
//!
//! # Crate Features
//!
//! None is on by default, and nothing reaches the operating system unless `std` is named; what
//! `std` adds is on 64-bit Linux and macOS.
//!
//! | Feature       | Adds                                                                       |
//! | ------------- | -------------------------------------------------------------------------- |
//! | `std`         | `SystemTime` conversions                                                   |
//! | `serde`       | every value's spelling, and the `serde` modules for counts in a named unit |
//! | `schemars`    | `JsonSchema` for every value with a spelling; turns `serde` on             |
//! | `zerocopy-08` | the zerocopy traits each type can honour, native-endian                    |
//! | `chrono-04`   | conversions to and from chrono 0.4's `DateTime` and `TimeDelta`            |
//! | `jiff-02`     | conversions to and from jiff 0.2's `Timestamp` and `SignedDuration`        |
//! | `time-03`     | conversions to and from time 0.3's `OffsetDateTime` and `Duration`         |

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(test, feature = "schemars"))]
extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

mod consts;
mod errors;
mod interop;
mod ops;
mod point;
mod rfc3339;
#[cfg(feature = "serde")]
pub mod serde;
mod spelling;
mod tick;
mod tick_rate;
mod timed;
mod timedelta;
mod timestamp;
mod uptime;
mod utc;

pub use crate::errors::{
    OutOfRangeError, ParseTaiTimestampError, ParseTickRateError, ParseTickdeltaError,
    ParseTimedeltaError, ParseTimestampError,
};
pub use crate::point::TimePoint;
pub use crate::tick::{Tickdelta, Tickstamp};
pub use crate::tick_rate::TickRate;
pub use crate::timed::Timed;
pub use crate::timedelta::Timedelta;
pub use crate::timestamp::{TaiTimestamp, Timestamp};
pub use crate::uptime::{BootUptime, RawUptime, Uptime};
pub use crate::utc::UtcDateTime;
