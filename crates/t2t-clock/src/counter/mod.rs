//! The CPU's counter: one instruction to read.

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(target_arch = "x86_64", test))]
mod tsc;
#[cfg(target_arch = "x86_64")]
mod x86_64;

use core::ops::RangeInclusive;

use t2t_core::{Tick, TickRate, Ticks, Timedelta};

#[cfg(target_arch = "aarch64")]
use crate::counter::aarch64::{discover_rate, read};
#[cfg(target_arch = "x86_64")]
use crate::counter::x86_64::{discover_rate, read};
use crate::{Clock, CounterError};

/// The rates a counter is believed at: below a tick a microsecond it measures nothing worth
/// measuring, and none runs past 10 GHz.
const PLAUSIBLE_RATES: RangeInclusive<u64> = 1_000_000..=10_000_000_000;

/// The CPU's free-running counter: `cntvct_el0` on `aarch64`, the time-stamp counter on `x86_64`.
///
/// A reading is one instruction that touches no memory. Every core and process on a machine reads
/// the same counter, so a [`Tick`] one process took, another may subtract from. Only the
/// difference of two readings means anything, and the counter's [`rate`](Self::rate) turns it into
/// a [`Timedelta`].
///
/// The read is not ordered against the instructions around it: right for stamping, and for timing
/// anything much longer than the read itself.
///
/// # Examples
/// ```
/// use t2t_clock::{Clock, Counter};
/// use t2t_core::Timedelta;
///
/// let counter = Counter::discover()?;
/// let start = counter.now();
/// let took = counter.timedelta(counter.now() - start);
///
/// assert!(took < Timedelta::from_millis(1), "two reads back to back");
/// # Ok::<(), t2t_clock::CounterError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counter {
    /// Ticks a second.
    rate: TickRate,
}

impl Counter {
    /// The counter, at the rate the CPU reports for it; where it reports none, on `x86_64` with
    /// `std` on 64-bit Linux or macOS, one measured against the OS clocks over 10 ms.
    ///
    /// # Errors
    /// - [`CounterError::NotInvariant`], the time-stamp counter's rate follows the core's.
    /// - [`CounterError::NoRate`], the CPU reports no rate, and none could be measured.
    /// - [`CounterError::ImplausibleRate`], the rate is outside 1 MHz to 10 GHz.
    pub fn discover() -> Result<Self, CounterError> {
        discover_rate().map(Self::with_rate)
    }

    /// The counter, at `rate`: one discovered elsewhere, or known.
    #[must_use]
    pub const fn with_rate(rate: TickRate) -> Self {
        Self { rate }
    }

    /// Ticks a second.
    #[inline]
    #[must_use]
    pub const fn rate(&self) -> TickRate {
        self.rate
    }

    /// How long `ticks` lasts at the counter's rate.
    #[inline]
    #[must_use]
    pub const fn timedelta(&self, ticks: Ticks) -> Timedelta {
        self.rate.timedelta(ticks)
    }
}

impl Clock for Counter {
    type Instant = Tick;

    #[inline(always)]
    #[expect(clippy::inline_always, reason = "a call around the read would move it")]
    fn now(&self) -> Tick {
        Tick::new(read())
    }
}

/// `ticks_per_second` as a rate, refused outside what any counter runs at.
fn plausible_rate(ticks_per_second: u64) -> Result<TickRate, CounterError> {
    TickRate::new(ticks_per_second)
        .filter(|_| PLAUSIBLE_RATES.contains(&ticks_per_second))
        .ok_or(CounterError::ImplausibleRate { ticks_per_second })
}

#[cfg(test)]
mod tests {
    use core::hint::black_box;

    use t2t_core::{TickRate, Ticks, Timedelta};

    use super::{PLAUSIBLE_RATES, plausible_rate};
    use crate::{Clock, Counter, CounterError};

    #[test]
    fn a_rate_outside_the_plausible_range_is_refused() {
        assert_eq!(plausible_rate(0), Err(CounterError::ImplausibleRate { ticks_per_second: 0 }));
        assert_eq!(
            plausible_rate(999_999),
            Err(CounterError::ImplausibleRate { ticks_per_second: 999_999 })
        );
    }

    #[test]
    fn a_rate_a_counter_runs_at_is_taken() {
        assert_eq!(
            plausible_rate(24_000_000),
            Ok(TickRate::new(24_000_000).expect("a nonzero rate"))
        );
        assert_eq!(plausible_rate(1_000_000_000), Ok(TickRate::GIGAHERTZ));
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri cannot execute the counter's read")]
    fn the_discovered_counter_runs_at_a_plausible_rate_and_advances() {
        let counter = Counter::discover().expect("a counter with a rate");
        assert!(
            PLAUSIBLE_RATES.contains(&counter.rate().get()),
            "a plausible rate: {}",
            counter.rate()
        );

        let start = counter.now();
        for value in 0_u64..10_000 {
            black_box(value);
        }
        let span = counter.now() - start;
        assert!(span >= Ticks::ZERO, "the counter never runs backwards: {span:?}");
        assert!(counter.timedelta(span) < Timedelta::from_millis(10), "ten thousand loops");
    }
}
