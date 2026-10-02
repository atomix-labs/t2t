//! The time-stamp counter, read with `rdtsc`.

use core::arch::x86_64::_rdtsc;

use raw_cpuid::CpuId;
use t2t_core::TickRate;

use crate::CounterError;
use crate::counter::{plausible_rate, tsc};

/// The time-stamp counter.
#[inline(always)]
#[expect(clippy::inline_always, reason = "a call around the read would move it")]
pub(super) fn read() -> i64 {
    // SAFETY: `_rdtsc` has no precondition: every `x86_64` CPU has `rdtsc`, and it needs no target
    // feature.
    #[expect(unsafe_code, reason = "`_rdtsc` is an `unsafe` intrinsic")]
    let ticks = unsafe { _rdtsc() };
    ticks.cast_signed()
}

/// The counter's rate, from CPUID, or measured where CPUID reports none.
pub(super) fn discover_rate() -> Result<TickRate, CounterError> {
    plausible_rate(tsc::ticks_per_second(&CpuId::new(), measure)?)
}

/// The counter's rate, measured over 10 ms against the monotonic clock at the hardware's rate.
#[cfg(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
))]
fn measure() -> Option<u64> {
    use core::hint;

    use t2t_core::Timedelta;

    use crate::{Clock, RawMonotonicClock};

    /// How long the measurement runs.
    const WINDOW: Timedelta = Timedelta::from_millis(10);

    let start_ticks = read();
    let start = RawMonotonicClock.now();
    let mut span = Timedelta::ZERO;
    while span < WINDOW {
        hint::spin_loop();
        span = RawMonotonicClock.now() - start;
    }
    let ticks = read().saturating_sub(start_ticks);
    let ticks_per_second =
        i128::from(ticks).checked_mul(1_000_000_000)?.checked_div(i128::from(span.as_nanos()))?;
    u64::try_from(ticks_per_second).ok()
}

/// No clock to measure against.
#[cfg(not(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
)))]
const fn measure() -> Option<u64> {
    None
}
