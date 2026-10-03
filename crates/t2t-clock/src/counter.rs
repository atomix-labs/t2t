//! The CPU's counter: one instruction to read.

use core::ops::RangeInclusive;

use arch::{discover_rate, read};
use t2t_core::{TickRate, Tickstamp};

use crate::{Clock, CounterError};

/// The rates a counter is believed at: below a tick a microsecond it measures nothing worth
/// measuring, and none runs past 10 GHz.
const PLAUSIBLE_RATES: RangeInclusive<u64> = 1_000_000..=10_000_000_000;

/// The CPU's free-running counter: `cntvct_el0` on `aarch64`, the time-stamp counter on `x86_64`.
///
/// A reading is one instruction that touches no memory. Every core and process on a machine reads
/// the same counter, so a [`Tickstamp`] one process took, another may subtract from. Only the
/// difference of two readings means anything, and the counter's [`rate`](Self::rate) turns it into
/// a span.
///
/// The read is not ordered against the instructions around it: right for stamping, and for timing
/// anything much longer than the read itself.
///
/// # Examples
/// ```
/// use t2t_clock::{Clock, Counter};
/// use t2t_core::Timedelta;
///
/// # // The CI's Intel macOS virtual machines promise no invariant counter, so have none to read.
/// # if Counter::discover() == Err(t2t_clock::CounterError::NotInvariant) {
/// #     return Ok(());
/// # }
/// let counter = Counter::discover()?;
/// let start = counter.now();
/// let took = (counter.now() - start).to_timedelta(counter.rate());
///
/// assert!(took < Timedelta::from_millis(1), "two reads back to back");
/// # Ok::<(), t2t_clock::CounterError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        discover_rate().map(Self::new)
    }

    /// The counter, at `rate`: one discovered elsewhere, or known.
    #[inline]
    #[must_use]
    pub const fn new(rate: TickRate) -> Self {
        Self { rate }
    }

    /// The counter's rate.
    #[inline]
    #[must_use]
    pub const fn rate(self) -> TickRate {
        self.rate
    }
}

impl Clock for Counter {
    type Reading = Tickstamp;

    #[inline(always)]
    #[expect(clippy::inline_always, reason = "a call around the read would move it")]
    fn now(&self) -> Tickstamp {
        Tickstamp::from_ticks(read())
    }
}

/// `hertz` as a rate, refused outside what any counter runs at.
///
/// # Errors
/// [`CounterError::ImplausibleRate`], `hertz` is outside 1 MHz to 10 GHz.
fn plausible_rate(hertz: u64) -> Result<TickRate, CounterError> {
    TickRate::from_hertz(hertz)
        .filter(|_| PLAUSIBLE_RATES.contains(&hertz))
        .ok_or(CounterError::ImplausibleRate { hertz })
}

/// The Arm generic timer's virtual counter, read from user space.
///
/// Linux and macOS let user space read both registers here; an OS that does not traps the read,
/// which stops the process but is no unsoundness.
#[cfg(target_arch = "aarch64")]
mod arch {
    use core::arch::asm;

    use t2t_core::TickRate;

    use super::plausible_rate;
    use crate::CounterError;

    /// The virtual counter, `cntvct_el0`.
    #[inline(always)]
    #[expect(clippy::inline_always, reason = "a call around the read would move it")]
    pub(super) fn read() -> i64 {
        let ticks: u64;
        // SAFETY: `mrs` writes `ticks` alone and touches no memory, stack or flags, as the options
        // say.
        #[expect(unsafe_code, reason = "a system register is read only through inline assembly")]
        unsafe {
            asm!(
                "mrs {ticks}, cntvct_el0",
                ticks = out(reg) ticks,
                options(nomem, nostack, preserves_flags),
            );
        }
        ticks.cast_signed()
    }

    /// The counter's rate, `cntfrq_el0`, which firmware writes and so is checked.
    ///
    /// # Errors
    /// [`CounterError::ImplausibleRate`], the register holds a rate outside 1 MHz to 10 GHz.
    pub(super) fn discover_rate() -> Result<TickRate, CounterError> {
        let hertz: u64;
        // SAFETY: `mrs` writes `hertz` alone and touches no memory, stack or flags, as the options
        // say.
        #[expect(unsafe_code, reason = "a system register is read only through inline assembly")]
        unsafe {
            asm!(
                "mrs {hertz}, cntfrq_el0",
                hertz = out(reg) hertz,
                options(nomem, nostack, preserves_flags),
            );
        }
        plausible_rate(hertz)
    }
}

/// The time-stamp counter, read with `rdtsc`, at the rate CPUID reports or one measured.
#[cfg(target_arch = "x86_64")]
mod arch {
    use core::arch::x86_64::_rdtsc;
    #[cfg(os_clocks)]
    use core::hint;

    use raw_cpuid::CpuId;
    use t2t_core::TickRate;
    #[cfg(os_clocks)]
    use t2t_core::Timedelta;

    use super::{plausible_rate, tsc};
    use crate::CounterError;
    #[cfg(os_clocks)]
    use crate::{Clock, RawMonotonicClock};

    /// The time-stamp counter.
    #[inline(always)]
    #[expect(clippy::inline_always, reason = "a call around the read would move it")]
    pub(super) fn read() -> i64 {
        // SAFETY: `_rdtsc` has no precondition: every `x86_64` CPU has `rdtsc`, and it needs no
        // target feature.
        #[expect(unsafe_code, reason = "`_rdtsc` is an `unsafe` intrinsic")]
        let ticks = unsafe { _rdtsc() };
        ticks.cast_signed()
    }

    /// The counter's rate, from CPUID, or measured where CPUID reports none.
    ///
    /// # Errors
    /// What [`tsc::hertz`] refuses, and a rate outside 1 MHz to 10 GHz.
    pub(super) fn discover_rate() -> Result<TickRate, CounterError> {
        plausible_rate(tsc::hertz(&CpuId::new(), measure)?)
    }

    /// The counter's rate, measured over 10 ms against the monotonic clock at the hardware's rate.
    #[cfg(os_clocks)]
    fn measure() -> Option<u64> {
        /// How long the measurement runs.
        const WINDOW: Timedelta = Timedelta::from_millis(10);

        let start_ticks = read();
        let start = RawMonotonicClock.now();
        let mut span = Timedelta::ZERO;
        while span < WINDOW {
            hint::spin_loop();
            span = RawMonotonicClock.now() - start;
        }
        let ticks = i128::from(read().saturating_sub(start_ticks));
        let second = i128::from(Timedelta::SECOND.as_nanos());
        let hertz = ticks.checked_mul(second)?.checked_div(i128::from(span.as_nanos()))?;
        u64::try_from(hertz).ok()
    }

    /// No clock to measure against.
    #[cfg(not(os_clocks))]
    const fn measure() -> Option<u64> {
        None
    }
}

/// What CPUID says of the time-stamp counter: whether it is invariant, and its rate.
#[cfg(any(target_arch = "x86_64", test))]
mod tsc {
    use raw_cpuid::{CpuId, CpuIdReader};

    use crate::CounterError;

    /// Hertz in a kilohertz, the unit a hypervisor reports the counter's rate in.
    const HERTZ_PER_KILOHERTZ: u64 = 1_000;

    /// The counter's ticks a second, refused where the counter is not invariant.
    ///
    /// The rate is the CPU's, from leaf `0x15`; else the hypervisor's, from leaf `0x4000_0010`;
    /// else the one `measure` takes.
    ///
    /// # Errors
    /// - [`CounterError::NotInvariant`], the counter's rate follows the core's.
    /// - [`CounterError::NoRate`], neither CPUID nor `measure` gives a rate.
    pub(super) fn hertz<R, M>(cpuid: &CpuId<R>, measure: M) -> Result<u64, CounterError>
    where
        R: CpuIdReader,
        M: FnOnce() -> Option<u64>,
    {
        let is_invariant =
            cpuid.get_advanced_power_mgmt_info().is_some_and(|power| power.has_invariant_tsc());
        if !is_invariant {
            return Err(CounterError::NotInvariant);
        }
        let hypervisor_rate = || {
            let kilohertz = cpuid.get_hypervisor_info()?.tsc_frequency()?;
            u64::from(kilohertz).checked_mul(HERTZ_PER_KILOHERTZ)
        };
        cpuid
            .get_tsc_info()
            .and_then(|leaf| leaf.tsc_frequency())
            .or_else(hypervisor_rate)
            .filter(|&hertz| hertz > 0)
            .or_else(measure)
            .ok_or(CounterError::NoRate)
    }

    #[cfg(test)]
    mod tests {
        use raw_cpuid::{CpuId, CpuIdReader, CpuIdResult};

        use super::hertz;
        use crate::CounterError;

        /// The highest basic and extended leaves, and the power leaf saying the counter is
        /// invariant.
        const INVARIANT: [(u32, [u32; 4]); 3] = [
            (0, [0x15, 0, 0, 0]),
            (0x8000_0000, [0x8000_0007, 0, 0, 0]),
            (0x8000_0007, [0, 0, 0, 1 << 8]),
        ];

        /// A rate leaf reporting a 38.4 MHz crystal, and a ratio of 176 to 2 to it.
        const RATE_LEAF: (u32, [u32; 4]) = (0x15, [2, 176, 38_400_000, 0]);

        /// A CPU answering each of `leaves` with its EAX, EBX, ECX and EDX, and every other with
        /// zeros.
        fn cpu<const COUNT: usize>(leaves: [(u32, [u32; 4]); COUNT]) -> CpuId<impl CpuIdReader> {
            CpuId::with_cpuid_reader(move |leaf, _subleaf| {
                let answer = leaves.iter().find(|&&(number, _)| number == leaf);
                let [eax, ebx, ecx, edx] = answer.map_or([0; 4], |&(_, registers)| registers);
                CpuIdResult { eax, ebx, ecx, edx }
            })
        }

        /// No measurement, as without `std`.
        const fn no_measurement() -> Option<u64> {
            None
        }

        #[test]
        fn an_invariant_counter_runs_at_the_rate_its_leaf_reports() {
            let [highest, extended, power] = INVARIANT;
            let cpuid = cpu([highest, extended, power, RATE_LEAF]);
            assert_eq!(hertz(&cpuid, no_measurement), Ok(3_379_200_000));
        }

        #[test]
        fn a_counter_that_is_not_invariant_is_refused() {
            let [highest, extended, _] = INVARIANT;
            let variable_rate = cpu([highest, extended, RATE_LEAF]);
            assert_eq!(hertz(&variable_rate, no_measurement), Err(CounterError::NotInvariant));

            let [highest, _, power] = INVARIANT;
            let old_cpu = cpu([highest, (0x8000_0000, [0x8000_0001, 0, 0, 0]), power, RATE_LEAF]);
            assert_eq!(hertz(&old_cpu, no_measurement), Err(CounterError::NotInvariant));
        }

        #[test]
        fn a_virtual_counter_runs_at_the_rate_its_hypervisor_reports() {
            let [_, extended, power] = INVARIANT;
            let cpuid = cpu([
                (0, [0x0D, 0, 0, 0]),
                (1, [0, 0, 1 << 31, 0]),
                extended,
                power,
                (0x4000_0000, [0x4000_0010, 0, 0, 0]),
                (0x4000_0010, [2_100_000, 0, 0, 0]),
            ]);
            assert_eq!(hertz(&cpuid, no_measurement), Ok(2_100_000_000));
        }

        #[test]
        fn a_counter_whose_cpu_reports_no_rate_is_measured_or_refused() {
            let silent_leaf = cpu(INVARIANT);
            assert_eq!(hertz(&silent_leaf, || Some(2_100_000_000)), Ok(2_100_000_000));
            assert_eq!(hertz(&silent_leaf, no_measurement), Err(CounterError::NoRate));

            let [highest, extended, power] = INVARIANT;
            let crystal_unknown = cpu([highest, extended, power, (0x15, [2, 168, 0, 0])]);
            assert_eq!(hertz(&crystal_unknown, no_measurement), Err(CounterError::NoRate));
        }
    }
}

#[cfg(test)]
mod tests {
    use core::hint::black_box;

    use t2t_core::{TickRate, Tickdelta, Timedelta};

    use super::{PLAUSIBLE_RATES, plausible_rate};
    use crate::{Clock, Counter, CounterError};

    #[test]
    fn a_rate_outside_the_plausible_range_is_refused() {
        let refused = |hertz| Err(CounterError::ImplausibleRate { hertz });
        assert_eq!(plausible_rate(0), refused(0), "a counter that never ticks");
        assert_eq!(plausible_rate(999_999), refused(999_999), "one too slow to measure by");
    }

    #[test]
    fn a_rate_a_counter_runs_at_is_taken() {
        let apple_silicon = TickRate::from_hertz(24_000_000).expect("a nonzero rate");
        assert_eq!(plausible_rate(24_000_000), Ok(apple_silicon), "Apple silicon's");
        assert_eq!(plausible_rate(1_000_000_000), Ok(TickRate::GIGAHERTZ), "and Armv8.6's");
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri cannot execute the counter's read")]
    fn the_counter_never_runs_backwards() {
        // A reading needs no rate.
        let counter = Counter::new(TickRate::GIGAHERTZ);
        let start = counter.now();
        for value in 0_u64..10_000 {
            black_box(value);
        }
        let span = counter.now() - start;
        assert!(span >= Tickdelta::ZERO, "the later reading is no earlier: {span}");
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri cannot execute the counter's read")]
    fn the_discovered_counter_runs_at_a_plausible_rate() {
        let discovered = Counter::discover();
        // An `x86_64` CPU, or a virtual machine's, may not promise an invariant time-stamp counter,
        // and may report no rate, which only the OS clocks can then measure: refusals `discover`
        // promises, which the CI's virtual machines give.
        let refused_by_the_cpu = match discovered {
            Err(CounterError::NotInvariant) => cfg!(target_arch = "x86_64"),
            Err(CounterError::NoRate) => cfg!(all(target_arch = "x86_64", not(os_clocks))),
            Ok(_) | Err(CounterError::ImplausibleRate { .. }) => false,
        };
        if refused_by_the_cpu {
            return;
        }
        let counter = discovered.expect("a counter with a rate");
        let rate = counter.rate();
        assert!(PLAUSIBLE_RATES.contains(&rate.as_hertz()), "a plausible rate: {rate}");

        let start = counter.now();
        for value in 0_u64..10_000 {
            black_box(value);
        }
        let took = (counter.now() - start).to_timedelta(rate);
        assert!(took < Timedelta::from_millis(10), "ten thousand loops in under 10 ms: {took}");
    }
}
