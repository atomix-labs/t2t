//! The Arm generic timer's virtual counter, read from user space.
//!
//! Linux and macOS let user space read both registers here; an OS that does not traps the read,
//! which stops the process but is no unsoundness.

use core::arch::asm;

use t2t_core::TickRate;

use crate::CounterError;
use crate::counter::plausible_rate;

/// The virtual counter, `cntvct_el0`.
#[inline(always)]
#[expect(clippy::inline_always, reason = "a call around the read would move it")]
pub(super) fn read() -> i64 {
    let ticks: u64;
    // SAFETY: `mrs` writes `ticks` alone and touches no memory, stack or flags, as the options say.
    #[expect(unsafe_code, reason = "a system register is read only through inline assembly")]
    unsafe {
        asm!("mrs {ticks}, cntvct_el0", ticks = out(reg) ticks, options(nomem, nostack, preserves_flags));
    }
    ticks.cast_signed()
}

/// The counter's rate, `cntfrq_el0`, which firmware writes and so is checked.
pub(super) fn discover_rate() -> Result<TickRate, CounterError> {
    let ticks_per_second: u64;
    // SAFETY: `mrs` writes `ticks_per_second` alone and touches no memory, stack or flags, as the
    // options say.
    #[expect(unsafe_code, reason = "a system register is read only through inline assembly")]
    unsafe {
        asm!(
            "mrs {ticks_per_second}, cntfrq_el0",
            ticks_per_second = out(reg) ticks_per_second,
            options(nomem, nostack, preserves_flags),
        );
    }
    plausible_rate(ticks_per_second)
}
