//! Time values and the clocks that read them, for code where a nanosecond counts.
//!
//! Each timeline has its own point and its own clocks, so a reading from one is never subtracted
//! from another's:
//!
//! ```text
//! point          timeline                                   read by
//! Timestamp      the wall clock, from the Unix epoch        SystemClock, CoarseSystemClock
//! TaiTimestamp   International Atomic Time, from 1970 TAI   TaiClock
//! Uptime         the monotonic clock, from near boot        MonotonicClock, CoarseMonotonicClock
//! RawUptime      the monotonic clock at the hardware's rate RawMonotonicClock
//! BootTime       the boot clock, counting suspensions       BootClock
//! Tick           a hardware counter                         Counter
//! Timedelta      the CPU time used, as a span               ProcessCpuClock, ThreadCpuClock
//! ```
//!
//! The span between two points is a `Timedelta`, or for a `Tick`, a count of `Ticks` that its
//! counter's `TickRate` turns into one.
//!
//! Every value is an `i64`, every operator saturates at the ends of the range, with a `checked_*`
//! twin, and every clock answers one verb, [`now`](clock::Clock::now). A workspace that denies
//! `clippy::arithmetic_side_effects` names the five points and spans in its clippy configuration's
//! `arithmetic-side-effects-allowed`.
//!
//! # Choosing a Clock
//!
//! - **Stamp a capture with `SystemClock`.** Its reading names a moment a venue and a peer machine
//!   name too. It may step backwards across a correction.
//! - **Wait on `MonotonicClock`.** It never steps, so a deadline is `MonotonicClock.now() +
//!   timeout`.
//! - **Measure with [`Counter`](clock::Counter).** One instruction that touches no memory, and one
//!   counter for every core and process on the machine.
//! - **Poll with `CoarseSystemClock` or `CoarseMonotonicClock`.** Either clock as the kernel's last
//!   tick left it, which reads no counter; never for a stamp.
//! - **Drive a test or a replay with [`ManualClock`](clock::ManualClock)**, or
//!   [`AtomicManualClock`](clock::AtomicManualClock) across threads.
//!
//! The system clocks need the `std` feature, on 64-bit Linux or macOS; [`clock`] has them all.
//!
//! ```
//! use t2t::clock::{Clock, ManualClock};
//! use t2t::{Timed, Timedelta, Timestamp};
//!
//! let clock = ManualClock::new(Timestamp::from_secs(1_700_000_000));
//! let quote = Timed::new(clock.now(), 101_u64);
//!
//! clock.advance(Timedelta::from_millis(1_500));
//! assert!(quote.elapsed(clock.now()) > Timedelta::SECOND, "stale a second and a half on");
//! ```
//!
//! # Crate features
//!
//! None is on by default, and nothing reaches the operating system unless `std` is named.
//!
//! | Feature     | Adds                                                                                 |
//! | ----------- | ------------------------------------------------------------------------------------ |
//! | `std`       | the system clocks, `SystemTime` conversions, and an `x86_64` counter's measured rate |
//! | `serde`     | the string spellings, and the `serde` modules for counts in a named unit             |
//! | `schemars`  | `JsonSchema` for `Timestamp` and `Timedelta`; turns `serde` on                       |
//! | `zerocopy`  | `FromBytes`, `IntoBytes` and the rest, where each type can honour them               |
//! | `chrono-04` | `Timestamp` and `Timedelta` to and from chrono 0.4's `DateTime` and `TimeDelta`      |
//! | `jiff-02`   | `Timestamp` and `Timedelta` to and from jiff 0.2's `Timestamp` and `SignedDuration`  |
//! | `time-03`   | `Timestamp` and `Timedelta` to and from time 0.3's `OffsetDateTime` and `Duration`   |

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(inline)]
pub use t2t_clock as clock;
#[cfg(feature = "serde")]
#[doc(inline)]
pub use t2t_core::serde;
pub use t2t_core::{
    BootTime, OutOfRangeError, ParseTimedeltaError, ParseTimestampError, RawUptime, TaiTimestamp,
    Tick, TickRate, Ticks, TimePoint, Timed, Timedelta, Timestamp, Uptime, UtcDateTime,
};
