//! What one reading of each clock costs, on one to eight threads at once.
//!
//! A clock whose reading shares a cache line with another core's would cost more as threads are
//! added; each here should not. A target without the operating system's clocks or a counter
//! benches what it has.
//!
//! ```text
//! cargo bench -p t2t-clock --features std --bench read
//! ```

#[cfg(counter)]
use std::sync::LazyLock;

use divan::main as run_benches;

fn main() {
    // The counter's rate is discovered before the first sample, which would otherwise time it.
    #[cfg(counter)]
    LazyLock::force(&counter::COUNTER);
    run_benches();
}

/// The operating system's clocks.
#[cfg(os_clocks)]
mod os {
    use divan::bench;
    #[cfg(tai_clock)]
    use t2t_clock::TaiClock;
    use t2t_clock::{
        BootClock, Clock, CoarseMonotonicClock, CoarseSystemClock, MonotonicClock, ProcessCpuClock,
        RawMonotonicClock, SystemClock, ThreadCpuClock,
    };
    #[cfg(tai_clock)]
    use t2t_core::TaiTimestamp;

    /// A reading of each clock both systems have: a clock is a unit struct, so building one with
    /// `C::default()` costs nothing, and a sample times the reading alone.
    #[bench(
        types = [
            SystemClock,
            CoarseSystemClock,
            MonotonicClock,
            CoarseMonotonicClock,
            RawMonotonicClock,
            BootClock,
            ProcessCpuClock,
            ThreadCpuClock,
        ],
        threads = [1, 2, 4, 8],
    )]
    fn read<C: Clock + Default>() -> C::Reading {
        C::default().now()
    }

    /// International Atomic Time, which Linux alone has a clock for.
    #[cfg(tai_clock)]
    #[bench(threads = [1, 2, 4, 8])]
    fn tai() -> TaiTimestamp {
        TaiClock.now()
    }
}

/// The CPU's counter.
#[cfg(counter)]
mod counter {
    use std::sync::LazyLock;

    use divan::bench;
    use t2t_clock::{Clock, Counter};
    use t2t_core::Tickstamp;

    /// The counter, its rate discovered once.
    #[expect(clippy::expect_used, reason = "a bench has no caller to hand a refused counter to")]
    pub(super) static COUNTER: LazyLock<Counter> =
        LazyLock::new(|| Counter::discover().expect("a counter with a rate"));

    /// A reading of the counter.
    #[bench(threads = [1, 2, 4, 8])]
    fn read() -> Tickstamp {
        COUNTER.now()
    }
}
