//! The listing of the chapter Getting Started.

#[test]
#[cfg_attr(miri, ignore = "Miri isolates the process from the OS's clocks")]
// ANCHOR: first
fn the_wall_clock_stamps_and_the_monotonic_clock_times() {
    use core::time::Duration;
    use std::thread;

    use t2t::Timedelta;
    use t2t::clock::{Clock, MonotonicClock, SystemClock};

    // A stamp another machine can read: the wall clock's.
    let captured = SystemClock.now();
    // A start that no correction to the wall clock moves: the monotonic clock's.
    let start = MonotonicClock.now();

    let pause = Timedelta::from_millis(20);
    thread::sleep(Duration::try_from(pause).expect("a span forwards"));

    let took = MonotonicClock.now() - start;
    println!("captured at {captured:.6}, then slept {took}");
    assert!(took >= pause, "at least the pause: {took}");
}
// ANCHOR_END: first
