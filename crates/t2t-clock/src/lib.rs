//! Clocks that read the time: the OS's wall and monotonic clocks, the CPU's counter, and
//! clocks set by hand.
//!
//! Each clock gives its reading through one verb, [`Clock::now`], and the reading's type says
//! which timeline it came from: subtracting an [`Uptime`] from a [`Timestamp`] does not compile.
//!
//! | Clock                  | Reads          | Steps?  | For                                      |
//! | ---------------------- | -------------- | ------- | ---------------------------------------- |
//! | `SystemClock`          | `Timestamp`    | yes     | a stamp other machines compare           |
//! | `CoarseSystemClock`    | `Timestamp`    | yes     | whether a heartbeat or expiry is due     |
//! | `TaiClock` (Linux)     | `TaiTimestamp` | yes     | a stamp on the timescale PTP keeps       |
//! | `MonotonicClock`       | `Uptime`       | never   | a deadline, a timeout                    |
//! | `CoarseMonotonicClock` | `Uptime`       | never   | a far deadline, polled often             |
//! | `RawMonotonicClock`    | `RawUptime`    | never   | a span no time service's slewing touches |
//! | `BootClock`            | `BootUptime`   | never   | a timeout that runs through a suspension |
//! | `ProcessCpuClock`      | `Timedelta`    | never   | the CPU time the process has used        |
//! | `ThreadCpuClock`       | `Timedelta`    | never   | the CPU time the calling thread has used |
//! | [`Counter`]            | [`Tickstamp`]  | never   | a stamp or a span in one instruction     |
//! | [`ManualClock`]        | any point      | by hand | a test or a replay on one thread         |
//! | [`AtomicManualClock`]  | any point      | by hand | a test or a replay shared across threads |
//!
//! The OS clocks need the `std` feature, on 64-bit Linux or macOS, and read the clock the
//! table's name says on each; each clock's docs give the ids. [`Counter`] reads the virtual counter
//! on `aarch64` and the time-stamp counter on `x86_64`, with or without `std`.
//!
//! # Examples
//! ```
//! use t2t_clock::{Clock, ManualClock};
//! use t2t_core::{Timedelta, Timestamp};
//!
//! /// Whether the quote stamped at `stamp` is older than a second.
//! fn is_stale<C: Clock<Reading = Timestamp>>(clock: &C, stamp: Timestamp) -> bool {
//!     clock.now() - stamp > Timedelta::SECOND
//! }
//!
//! let clock = ManualClock::new(Timestamp::from_secs(10));
//! assert!(!is_stale(&clock, Timestamp::from_secs(10)), "fresh at capture");
//! clock.advance(Timedelta::from_secs(2));
//! assert!(is_stale(&clock, Timestamp::from_secs(10)), "stale two seconds on");
//! ```
//!
//! # Crate Features
//!
//! None is on by default, and nothing reaches the operating system unless `std` is named; what
//! `std` adds is on 64-bit Linux and macOS.
//!
//! | Feature | Adds                                                   |
//! | ------- | ------------------------------------------------------ |
//! | `std`   | the OS clocks, and an `x86_64` counter's measured rate |
//!
//! [`Timestamp`]: t2t_core::Timestamp
//! [`Uptime`]: t2t_core::Uptime
//! [`Tickstamp`]: t2t_core::Tickstamp

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
// docs.rs names what the OS clocks and the counter need in words, as their re-exports below say
// it, and never by the aliases `build.rs` declares; AtomicManualClock, which builds wherever a
// 64-bit atomic does, shows none.
#![cfg_attr(docsrs, doc(auto_cfg(hide(os_clocks, tai_clock, counter, atomic_clock))))]

#[cfg(test)]
extern crate alloc;
#[cfg(test)]
extern crate std;

mod clock;
#[cfg(counter)]
mod counter;
#[cfg(counter)]
mod errors;
mod manual;
#[cfg(os_clocks)]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "std",
        target_pointer_width = "64",
        any(target_os = "linux", target_os = "macos")
    )))
)]
mod os;
#[cfg(atomic_clock)]
mod sync;

pub use crate::clock::Clock;
#[cfg(counter)]
#[cfg_attr(docsrs, doc(cfg(any(target_arch = "aarch64", target_arch = "x86_64"))))]
pub use crate::counter::Counter;
#[cfg(counter)]
#[cfg_attr(docsrs, doc(cfg(any(target_arch = "aarch64", target_arch = "x86_64"))))]
pub use crate::errors::CounterError;
#[cfg(atomic_clock)]
pub use crate::manual::AtomicManualClock;
pub use crate::manual::ManualClock;
#[cfg(tai_clock)]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "std", target_pointer_width = "64", target_os = "linux")))
)]
pub use crate::os::TaiClock;
#[cfg(os_clocks)]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "std",
        target_pointer_width = "64",
        any(target_os = "linux", target_os = "macos")
    )))
)]
pub use crate::os::{
    BootClock, CoarseMonotonicClock, CoarseSystemClock, MonotonicClock, ProcessCpuClock,
    RawMonotonicClock, SystemClock, ThreadCpuClock,
};
