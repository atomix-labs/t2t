//! What one reading of each clock costs, on one to eight threads at once.
//!
//! A clock whose reading shares a cache line with another core's would cost more as threads are
//! added; each here should not.
//!
//! ```text
//! cargo bench -p t2t-clock --features std --bench read
//! ```

#![expect(clippy::expect_used, reason = "a bench has no caller to hand a refused counter to")]

use std::sync::LazyLock;

#[cfg(target_os = "linux")]
use t2t_clock::TaiClock;
use t2t_clock::{
    BootClock, Clock, CoarseMonotonicClock, CoarseSystemClock, Counter, MonotonicClock,
    ProcessCpuClock, RawMonotonicClock, SystemClock, ThreadCpuClock,
};
#[cfg(target_os = "linux")]
use t2t_core::TaiTimestamp;
use t2t_core::{BootTime, RawUptime, Tick, Timedelta, Timestamp, Uptime};

/// The counter, its rate discovered once.
static COUNTER: LazyLock<Counter> =
    LazyLock::new(|| Counter::discover().expect("a counter with a rate"));

fn main() {
    LazyLock::force(&COUNTER);
    divan::main();
}

/// The wall clock.
#[divan::bench(threads = [1, 2, 4, 8])]
fn system() -> Timestamp {
    SystemClock.now()
}

/// The coarse wall clock.
#[divan::bench(threads = [1, 2, 4, 8])]
fn coarse_system() -> Timestamp {
    CoarseSystemClock.now()
}

/// International Atomic Time.
#[cfg(target_os = "linux")]
#[divan::bench(threads = [1, 2, 4, 8])]
fn tai() -> TaiTimestamp {
    TaiClock.now()
}

/// The monotonic clock.
#[divan::bench(threads = [1, 2, 4, 8])]
fn monotonic() -> Uptime {
    MonotonicClock.now()
}

/// The coarse monotonic clock.
#[divan::bench(threads = [1, 2, 4, 8])]
fn coarse_monotonic() -> Uptime {
    CoarseMonotonicClock.now()
}

/// The monotonic clock at the hardware's rate.
#[divan::bench(threads = [1, 2, 4, 8])]
fn raw_monotonic() -> RawUptime {
    RawMonotonicClock.now()
}

/// The boot clock.
#[divan::bench(threads = [1, 2, 4, 8])]
fn boot() -> BootTime {
    BootClock.now()
}

/// The process's CPU time.
#[divan::bench(threads = [1, 2, 4, 8])]
fn process_cpu() -> Timedelta {
    ProcessCpuClock.now()
}

/// The calling thread's CPU time.
#[divan::bench(threads = [1, 2, 4, 8])]
fn thread_cpu() -> Timedelta {
    ThreadCpuClock.now()
}

/// The CPU's counter.
#[divan::bench(threads = [1, 2, 4, 8])]
fn counter() -> Tick {
    COUNTER.now()
}
