//! The operating system's clocks, read through `clock_gettime`.

#[cfg(target_os = "linux")]
use t2t_core::TaiTimestamp;
use t2t_core::{BootTime, RawUptime, Timedelta, Timestamp, Uptime};

use crate::Clock;

/// The ids each clock reads on Linux.
#[cfg(target_os = "linux")]
mod ids {
    pub(super) use libc::{
        CLOCK_BOOTTIME as BOOT, CLOCK_MONOTONIC as MONOTONIC,
        CLOCK_MONOTONIC_COARSE as COARSE_MONOTONIC, CLOCK_MONOTONIC_RAW as RAW_MONOTONIC,
        CLOCK_PROCESS_CPUTIME_ID as PROCESS_CPU, CLOCK_REALTIME as SYSTEM,
        CLOCK_REALTIME_COARSE as COARSE_SYSTEM, CLOCK_TAI as TAI,
        CLOCK_THREAD_CPUTIME_ID as THREAD_CPU,
    };
}

/// The ids each clock reads on macOS, which has no coarse wall clock and no TAI; its uptime
/// clocks stop while the machine sleeps, and its `CLOCK_MONOTONIC` counts the sleep.
#[cfg(target_os = "macos")]
mod ids {
    pub(super) use libc::{
        CLOCK_MONOTONIC as BOOT, CLOCK_PROCESS_CPUTIME_ID as PROCESS_CPU,
        CLOCK_REALTIME as COARSE_SYSTEM, CLOCK_REALTIME as SYSTEM,
        CLOCK_THREAD_CPUTIME_ID as THREAD_CPU, CLOCK_UPTIME_RAW as MONOTONIC,
        CLOCK_UPTIME_RAW as RAW_MONOTONIC, CLOCK_UPTIME_RAW_APPROX as COARSE_MONOTONIC,
    };
}

/// Nanoseconds on the clock `id`, one of [`ids`], each a clock its OS has, so the call cannot
/// fail.
#[inline]
fn read(id: libc::clockid_t) -> i64 {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `time` is a valid, writable `timespec`, the only memory `clock_gettime` writes.
    #[expect(unsafe_code, reason = "`clock_gettime` is a C function")]
    unsafe {
        libc::clock_gettime(id, &raw mut time);
    }
    time.tv_sec.saturating_mul(1_000_000_000).saturating_add(time.tv_nsec)
}

/// A clock that reads the OS clock `$id` as a `$point`.
macro_rules! system_clock {
    ($(#[$attribute:meta])* $clock:ident, $point:ident, $id:expr) => {
        $(#[$attribute])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct $clock;

        impl Clock for $clock {
            type Instant = $point;

            #[inline]
            fn now(&self) -> $point {
                $point::from_nanos(read($id))
            }
        }
    };
}

system_clock!(
    /// The wall clock, `CLOCK_REALTIME`.
    ///
    /// The clock whose readings mean something off this machine, which makes it the stamp for a
    /// capture. A time service disciplines it, so it may step backwards across a correction; a
    /// span that must never run backwards is measured on [`MonotonicClock`] or a
    /// [`Counter`](crate::Counter).
    ///
    /// # Examples
    /// ```
    /// use t2t_clock::{Clock, SystemClock};
    /// use t2t_core::Timestamp;
    ///
    /// assert!(SystemClock.now() > Timestamp::from_secs(1_700_000_000), "past November 2023");
    /// ```
    SystemClock,
    Timestamp,
    ids::SYSTEM
);

system_clock!(
    /// The wall clock as the kernel's last tick left it, `CLOCK_REALTIME_COARSE`.
    ///
    /// The moment [`SystemClock`] names, to the kernel's tick, a few milliseconds: the kernel
    /// hands back a value its timer wrote, and reads no counter. Right for asking whether a
    /// heartbeat or an expiry is due; never for stamping a capture. macOS has no coarse wall
    /// clock, so there it reads the exact one.
    ///
    /// # Examples
    /// ```
    /// use t2t_clock::{Clock, CoarseSystemClock, SystemClock};
    /// use t2t_core::Timedelta;
    ///
    /// let gap = SystemClock.now() - CoarseSystemClock.now();
    /// assert!(gap.abs() < Timedelta::from_millis(100), "within a tick or two: {gap}");
    /// ```
    CoarseSystemClock,
    Timestamp,
    ids::COARSE_SYSTEM
);

#[cfg(target_os = "linux")]
system_clock!(
    /// International Atomic Time, `CLOCK_TAI`, on Linux.
    ///
    /// The wall clock plus the kernel's TAI offset, which `chrony` or `ptp4l` sets to the leap
    /// seconds since 1972; until one does, the offset is zero and this reads the wall clock.
    TaiClock,
    TaiTimestamp,
    ids::TAI
);

system_clock!(
    /// The monotonic clock: `CLOCK_MONOTONIC` on Linux, `CLOCK_UPTIME_RAW` on macOS.
    ///
    /// It never steps, and every process on the machine reads the same clock, which makes it the
    /// clock for a deadline. It stops while the machine sleeps. Linux lets a time service slew its
    /// rate; macOS does not.
    ///
    /// # Examples
    /// ```
    /// use t2t_clock::{Clock, MonotonicClock};
    /// use t2t_core::Timedelta;
    ///
    /// let deadline = MonotonicClock.now() + Timedelta::from_millis(50);
    /// assert!(MonotonicClock.now() < deadline, "the deadline lies ahead");
    /// ```
    MonotonicClock,
    Uptime,
    ids::MONOTONIC
);

system_clock!(
    /// The monotonic clock as the kernel's last tick left it: `CLOCK_MONOTONIC_COARSE` on Linux,
    /// `CLOCK_UPTIME_RAW_APPROX` on macOS.
    ///
    /// The instant [`MonotonicClock`] names, to the kernel's tick, a few milliseconds: right for a
    /// deadline that is far off, or polled often.
    CoarseMonotonicClock,
    Uptime,
    ids::COARSE_MONOTONIC
);

system_clock!(
    /// The monotonic clock at the hardware's own rate: `CLOCK_MONOTONIC_RAW` on Linux,
    /// `CLOCK_UPTIME_RAW` on macOS.
    ///
    /// No time service adjusts its rate, which makes it the clock to measure a counter's rate
    /// against, and a span free of a time service's corrections. It stops while the machine
    /// sleeps.
    RawMonotonicClock,
    RawUptime,
    ids::RAW_MONOTONIC
);

system_clock!(
    /// The boot clock, which counts the time the machine was suspended: `CLOCK_BOOTTIME` on Linux,
    /// `CLOCK_MONOTONIC` on macOS.
    ///
    /// Right for a timeout that must expire across a suspension, which [`MonotonicClock`] would
    /// carry past.
    BootClock,
    BootTime,
    ids::BOOT
);

system_clock!(
    /// The CPU time the process has used, on every thread, `CLOCK_PROCESS_CPUTIME_ID`.
    ///
    /// Read through a system call, not the fast path the clocks above take.
    ProcessCpuClock,
    Timedelta,
    ids::PROCESS_CPU
);

system_clock!(
    /// The CPU time the calling thread has used, `CLOCK_THREAD_CPUTIME_ID`.
    ///
    /// Read through a system call, not the fast path the clocks above take. Two readings on two
    /// threads count two different times.
    ///
    /// # Examples
    /// ```
    /// use core::hint::black_box;
    ///
    /// use t2t_clock::{Clock, ThreadCpuClock};
    /// use t2t_core::Timedelta;
    ///
    /// let start = ThreadCpuClock.now();
    /// for value in 0_u64..1_000_000 {
    ///     black_box(value);
    /// }
    /// assert!(ThreadCpuClock.now() - start > Timedelta::ZERO, "the loop took CPU time");
    /// ```
    ThreadCpuClock,
    Timedelta,
    ids::THREAD_CPU
);

#[cfg(test)]
mod tests {
    use core::hint::black_box;
    use core::time::Duration;
    use std::thread;

    use t2t_core::{Timedelta, Timestamp};

    use crate::{
        BootClock, Clock, CoarseMonotonicClock, CoarseSystemClock, MonotonicClock, ProcessCpuClock,
        RawMonotonicClock, SystemClock, ThreadCpuClock,
    };

    #[test]
    #[cfg_attr(miri, ignore = "Miri isolates the process from the system's clocks")]
    fn the_wall_clock_reads_a_plausible_moment() {
        let now = SystemClock.now();
        assert!(now > Timestamp::from_secs(1_700_000_000), "after November 2023: {now}");
        assert!(now < Timestamp::from_secs(4_102_444_800), "before 2100: {now}");
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri isolates the process from the system's clocks")]
    fn a_coarse_clock_trails_its_exact_one_by_under_a_tick() {
        let coarse = CoarseSystemClock.now();
        let exact = SystemClock.now();
        assert!(exact >= coarse, "the exact reading came second: {coarse} then {exact}");
        assert!(exact - coarse < Timedelta::from_millis(100), "within a tick or two");

        let coarse = CoarseMonotonicClock.now();
        let exact = MonotonicClock.now();
        assert!(exact >= coarse, "the exact reading came second: {coarse} then {exact}");
        assert!(exact - coarse < Timedelta::from_millis(100), "within a tick or two");
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri isolates the process from the system's clocks")]
    fn the_monotonic_clocks_advance_by_what_was_slept() {
        let (start, raw_start, boot_start) =
            (MonotonicClock.now(), RawMonotonicClock.now(), BootClock.now());
        thread::sleep(Duration::from_millis(20));
        for span in [
            MonotonicClock.now() - start,
            RawMonotonicClock.now() - raw_start,
            BootClock.now() - boot_start,
        ] {
            assert!(span >= Timedelta::from_millis(20), "at least what was slept: {span}");
            assert!(span < Timedelta::from_secs(5), "and not wildly more: {span}");
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[cfg_attr(miri, ignore = "Miri isolates the process from the system's clocks")]
    fn tai_runs_at_or_ahead_of_the_wall_clock() {
        let wall = SystemClock.now().as_nanos();
        let tai = crate::TaiClock.now().as_nanos();
        let offset = Timedelta::from_nanos(tai.saturating_sub(wall));
        assert!(offset >= Timedelta::ZERO, "no time service sets a negative offset: {offset}");
        assert!(offset < Timedelta::from_secs(60), "the leap seconds, at most: {offset}");
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri isolates the process from the system's clocks")]
    fn the_cpu_clocks_count_the_work_done() {
        let (process_start, thread_start) = (ProcessCpuClock.now(), ThreadCpuClock.now());
        for value in 0_u64..1_000_000 {
            black_box(value);
        }
        let thread_time = ThreadCpuClock.now() - thread_start;
        let process_time = ProcessCpuClock.now() - process_start;
        assert!(thread_time > Timedelta::ZERO, "the loop took CPU time");
        assert!(process_time >= thread_time, "the process counts its threads': {process_time}");
    }
}
