//! Time values and the clocks that read them, for code where a nanosecond counts.
//!
//! Each timeline has its own type and its own clocks, so a reading from one is never subtracted
//! from another's:
//!
//! | Reading          | Timeline                     | Read by                                  |
//! | ---------------- | ---------------------------- | ---------------------------------------- |
//! | [`Timestamp`]    | wall clock, from 1970 UTC    | `SystemClock`, `CoarseSystemClock`       |
//! | [`TaiTimestamp`] | TAI, from 1970 TAI           | `TaiClock`                               |
//! | [`Uptime`]       | monotonic clock, near boot   | `MonotonicClock`, `CoarseMonotonicClock` |
//! | [`RawUptime`]    | monotonic clock, unslewed    | `RawMonotonicClock`                      |
//! | [`BootUptime`]   | boot clock, with suspensions | `BootClock`                              |
//! | [`Tickstamp`]    | hardware counter             | [`Counter`](clock::Counter)              |
//! | [`Timedelta`]    | CPU time used, a span        | `ProcessCpuClock`, `ThreadCpuClock`      |
//!
//! The span between two points is a [`Timedelta`], or for a [`Tickstamp`], a [`Tickdelta`] that its
//! counter's [`TickRate`] turns into one.
//!
//! Every point and span is an `i64`, every operator saturates at the ends of the range, with a
//! `checked_*` twin, and every clock answers one verb, [`now`](clock::Clock::now). A workspace that
//! denies `clippy::arithmetic_side_effects` names each point and span in its clippy configuration's
//! `arithmetic-side-effects-allowed`.
//!
//! # Choosing a Clock
//!
//! - **Stamp a capture with `SystemClock`.** Its reading names a moment another machine names too.
//!   It may step backwards across a correction.
//! - **Wait on `MonotonicClock`.** It never steps, so a deadline is `MonotonicClock.now() +
//!   timeout`.
//! - **Measure with [`Counter`](clock::Counter).** One instruction that touches no memory, and one
//!   counter for every core and process on the machine.
//! - **Poll with `CoarseSystemClock` or `CoarseMonotonicClock`.** Either clock as the kernel's
//!   timer last left it, which reads no counter; never for a stamp.
//! - **Drive a test or a replay with [`ManualClock`](clock::ManualClock)**, or
//!   [`AtomicManualClock`](clock::AtomicManualClock) across threads.
//!
//! The OS clocks need the `std` feature, on 64-bit Linux or macOS; [`clock`] has them all.
//!
//! # Examples
//! ```
//! use t2t::clock::{Clock, ManualClock};
//! use t2t::{Timed, Timedelta, Timestamp};
//!
//! let clock = ManualClock::new(Timestamp::from_secs(1_700_000_000));
//! let sample = Timed::new(clock.now(), 101_u64);
//!
//! clock.advance(Timedelta::from_millis(1_500));
//! assert!(sample.elapsed(clock.now()) > Timedelta::SECOND, "stale a second and a half on");
//! ```
//!
//! # Crate Features
//!
//! None is on by default, and nothing reaches the operating system unless `std` is named; what
//! `std` adds is on 64-bit Linux and macOS.
//!
//! | Feature     | Adds                                                                         |
//! | ----------- | ---------------------------------------------------------------------------- |
//! | `std`       | the OS clocks, an `x86_64` counter's measured rate, `SystemTime` conversions |
//! | `serde`     | every value's spelling, and the `serde` modules for counts in a named unit   |
//! | `schemars`  | `JsonSchema` for every value with a spelling; turns `serde` on               |
//! | `zerocopy`  | the zerocopy traits each type can honour, native-endian                      |
//! | `chrono-04` | conversions to and from chrono 0.4's `DateTime` and `TimeDelta`              |
//! | `jiff-02`   | conversions to and from jiff 0.2's `Timestamp` and `SignedDuration`          |
//! | `time-03`   | conversions to and from time 0.3's `OffsetDateTime` and `Duration`           |

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(inline)]
pub use t2t_clock as clock;
#[cfg(feature = "serde")]
#[doc(inline)]
pub use t2t_core::serde;
pub use t2t_core::{
    BootUptime, OutOfRangeError, ParseTaiTimestampError, ParseTickRateError, ParseTickdeltaError,
    ParseTimedeltaError, ParseTimestampError, RawUptime, TaiTimestamp, TickRate, Tickdelta,
    Tickstamp, TimePoint, Timed, Timedelta, Timestamp, Uptime, UtcDateTime,
};
