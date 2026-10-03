//! The listings of the chapter Choosing a Clock.

#[test]
#[cfg_attr(miri, ignore = "Miri isolates the process from the OS's clocks")]
// ANCHOR: stamp
fn the_wall_clock_stamps_a_capture() {
    use t2t::clock::{Clock, CoarseSystemClock, SystemClock};
    use t2t::{Timedelta, Timestamp};

    let captured = SystemClock.now();
    assert!(captured > Timestamp::from_secs(1_700_000_000), "a moment past November 2023");
    println!("captured at {captured}");

    // The coarse clock names the same moment, to the kernel's timer period.
    let gap = SystemClock.now() - CoarseSystemClock.now();
    assert!(gap.abs() < Timedelta::from_millis(100), "within a period or two: {gap}");
}
// ANCHOR_END: stamp

#[test]
#[cfg_attr(miri, ignore = "Miri isolates the process from the OS's clocks")]
// ANCHOR: deadline
fn the_monotonic_clock_keeps_a_deadline() {
    use core::time::Duration;
    use std::thread;

    use t2t::Timedelta;
    use t2t::clock::{Clock, CoarseMonotonicClock, MonotonicClock};

    // Sleep for what is left until the deadline, however early a sleep returns.
    let deadline = MonotonicClock.now() + Timedelta::from_millis(5);
    let mut left = deadline - MonotonicClock.now();
    while left.is_positive() {
        thread::sleep(Duration::try_from(left).expect("a span forwards is a duration"));
        left = deadline - MonotonicClock.now();
    }

    // A deadline far off, checked often, is cheaper on the coarse clock: both read `Uptime`.
    let expiry = CoarseMonotonicClock.now() + Timedelta::from_secs(30);
    assert!(MonotonicClock.now() < expiry, "the two read one timeline, so they compare");
}
// ANCHOR_END: deadline

#[test]
#[cfg_attr(miri, ignore = "Miri isolates the process from the OS's clocks")]
// ANCHOR: raw-and-boot
fn the_raw_and_boot_clocks_read_timelines_of_their_own() {
    use t2t::Timedelta;
    use t2t::clock::{BootClock, Clock, RawMonotonicClock};

    // A span no time service's slewing bends.
    let start = RawMonotonicClock.now();
    let span = RawMonotonicClock.now() - start;
    assert!(span >= Timedelta::ZERO, "it never runs backwards");

    // A lease that a suspension of the machine runs down too.
    let lease = BootClock.now() + Timedelta::from_mins(5);
    assert!(BootClock.now() < lease, "the lease lies ahead");
}
// ANCHOR_END: raw-and-boot

#[test]
#[cfg_attr(miri, ignore = "Miri isolates the process from the OS's clocks")]
// ANCHOR: cpu
fn the_cpu_clocks_count_the_work_done() {
    use core::hint::black_box;

    use t2t::Timedelta;
    use t2t::clock::{Clock, ProcessCpuClock, ThreadCpuClock};

    // The process's clock is read first and last, so its span holds the thread's.
    let (process_start, thread_start) = (ProcessCpuClock.now(), ThreadCpuClock.now());
    for value in 0_u64..1_000_000 {
        black_box(value);
    }
    let thread_time = ThreadCpuClock.now() - thread_start;
    let process_time = ProcessCpuClock.now() - process_start;
    assert!(thread_time > Timedelta::ZERO, "the loop took this thread's time: {thread_time}");
    assert!(process_time >= thread_time, "and the process counts every thread's: {process_time}");
}
// ANCHOR_END: cpu

#[test]
#[cfg(target_os = "linux")]
#[cfg_attr(miri, ignore = "Miri isolates the process from the OS's clocks")]
// ANCHOR: tai
fn the_tai_clock_runs_ahead_of_the_wall_clock_by_the_kernels_offset() {
    use t2t::Timedelta;
    use t2t::clock::{Clock, SystemClock, TaiClock};

    // The two are different timelines, so their counts are compared, not the points.
    let wall = SystemClock.now().as_nanos();
    let tai = TaiClock.now().as_nanos();
    let offset = Timedelta::from_nanos(tai.saturating_sub(wall)).floor(Timedelta::SECOND);
    println!("TAI runs {offset} ahead of UTC here");
    let leap_seconds = Timedelta::ZERO..Timedelta::from_secs(60);
    assert!(leap_seconds.contains(&offset), "zero, or the leap seconds since 1972: {offset}");
}
// ANCHOR_END: tai
