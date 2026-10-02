//! Clocks that read the time: the system's wall and monotonic clocks, the CPU's counter, and
//! clocks set by hand.
//!
//! Each clock reads its own kind of point through one verb, [`Clock::now`], and the point says
//! which timeline a value came from: subtracting an [`Uptime`] from a [`Timestamp`] does not
//! compile.
//!
//! | Clock                  | Reads          | Steps?  | For                                          |
//! | ---------------------- | -------------- | ------- | -------------------------------------------- |
//! | `SystemClock`          | `Timestamp`    | yes     | stamping a capture other machines compare    |
//! | `CoarseSystemClock`    | `Timestamp`    | yes     | asking whether a heartbeat or expiry is due  |
//! | `TaiClock` (Linux)     | `TaiTimestamp` | never   | a stamp on the timescale PTP keeps           |
//! | `MonotonicClock`       | `Uptime`       | never   | a deadline, a timeout                        |
//! | `CoarseMonotonicClock` | `Uptime`       | never   | a far deadline, polled often                 |
//! | `RawMonotonicClock`    | `RawUptime`    | never   | a span no time service's slewing touches     |
//! | `BootClock`            | `BootTime`     | never   | a timeout that runs on through a suspension  |
//! | `ProcessCpuClock`      | `Timedelta`    | never   | the CPU time the process has used            |
//! | `ThreadCpuClock`       | `Timedelta`    | never   | the CPU time the calling thread has used     |
//! | [`Counter`]            | [`Tick`]       | never   | a stamp or a span in one instruction         |
//! | [`ManualClock`]        | any point      | by hand | a test or a replay on one thread             |
//! | [`AtomicManualClock`]  | any point      | by hand | a test or a replay shared across threads     |
//!
//! The system clocks need the `std` feature, on 64-bit Linux or macOS, and read the clock the
//! table's name says on each; each clock's docs give the ids. [`Counter`] reads the virtual counter
//! on `aarch64` and the time-stamp counter on `x86_64`, with or without `std`.
//!
//! ```
//! use t2t_clock::{Clock, ManualClock};
//! use t2t_core::{Timedelta, Timestamp};
//!
//! /// Whether the quote stamped at `stamp` is older than a second.
//! fn is_stale<C: Clock<Instant = Timestamp>>(clock: &C, stamp: Timestamp) -> bool {
//!     clock.now() - stamp > Timedelta::SECOND
//! }
//!
//! let clock = ManualClock::new(Timestamp::from_secs(10));
//! assert!(!is_stale(&clock, Timestamp::from_secs(10)), "fresh at capture");
//! clock.advance(Timedelta::from_secs(2));
//! assert!(is_stale(&clock, Timestamp::from_secs(10)), "stale two seconds on");
//! ```
//!
//! # Crate features
//!
//! | Feature | Adds                                                                                   |
//! | ------- | -------------------------------------------------------------------------------------- |
//! | `std`   | the system clocks, and measuring an `x86_64` counter's rate                            |
//!
//! [`Timestamp`]: t2t_core::Timestamp
//! [`Uptime`]: t2t_core::Uptime
//! [`Tick`]: t2t_core::Tick

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
extern crate std;

mod clock;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
mod counter;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
mod errors;
mod manual;
#[cfg(target_has_atomic = "64")]
mod sync;
#[cfg(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
))]
mod system;

pub use crate::clock::Clock;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub use crate::counter::Counter;
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub use crate::errors::CounterError;
#[cfg(target_has_atomic = "64")]
pub use crate::manual::AtomicManualClock;
pub use crate::manual::ManualClock;
#[cfg(all(feature = "std", target_pointer_width = "64", target_os = "linux"))]
pub use crate::system::TaiClock;
#[cfg(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
))]
pub use crate::system::{
    BootClock, CoarseMonotonicClock, CoarseSystemClock, MonotonicClock, ProcessCpuClock,
    RawMonotonicClock, SystemClock, ThreadCpuClock,
};
