//! The listings of the chapter Testing with Manual Clocks.

#[test]
// ANCHOR: generic
fn code_generic_over_its_clock_is_tested_on_a_manual_one() {
    use t2t::clock::{Clock, ManualClock};
    use t2t::{Timed, Timedelta, Timestamp};

    /// Whether `sample` is older than a second by `clock`: `SystemClock` in production.
    fn is_stale<C: Clock<Reading = Timestamp>>(clock: &C, sample: Timed<u64>) -> bool {
        sample.elapsed(clock.now()) > Timedelta::SECOND
    }

    let clock = ManualClock::new(Timestamp::from_secs(1_700_000_000));
    let sample = Timed::new(clock.now(), 101);
    assert!(!is_stale(&clock, sample), "fresh when captured");
    clock.advance(Timedelta::from_millis(1_500));
    assert!(is_stale(&clock, sample), "stale a second and a half on");
}
// ANCHOR_END: generic

#[test]
// ANCHOR: manual
fn a_manual_clock_moves_only_when_moved() {
    use t2t::clock::{Clock, ManualClock};
    use t2t::{Timedelta, Uptime};

    let clock = ManualClock::new(Uptime::from_secs(10));
    assert_eq!(clock.now(), clock.now(), "it stands still between moves");

    clock.advance(Timedelta::from_millis(250));
    assert_eq!(clock.now(), Uptime::from_millis(10_250), "forwards by a span");
    clock.advance(-Timedelta::SECOND);
    assert_eq!(clock.now(), Uptime::from_millis(9_250), "backwards by a negative one");
    clock.set(Uptime::MAX);
    clock.advance(Timedelta::SECOND);
    assert_eq!(clock.now(), Uptime::MAX, "saturating at the end, as the point's addition does");
}
// ANCHOR_END: manual

#[test]
// ANCHOR: replay
fn a_replay_sets_the_clock_to_each_recorded_stamp() {
    use t2t::clock::{Clock, ManualClock};
    use t2t::{Timedelta, Timestamp};

    /// Whether a heartbeat last seen at `last` has lapsed by `clock`, past `limit`.
    fn has_lapsed<C: Clock<Reading = Timestamp>>(
        clock: &C, last: Timestamp, limit: Timedelta,
    ) -> bool {
        clock.now() - last > limit
    }

    // The stamps a capture recorded, one a heartbeat.
    let recorded = ["2026-09-16T07:45:35Z", "2026-09-16T07:45:36.5Z", "2026-09-16T07:45:40Z"];
    let beats: Vec<Timestamp> =
        recorded.iter().map(|text| text.parse().expect("an instant")).collect();

    let clock = ManualClock::new(beats[0]);
    let mut lapses = Vec::new();
    for pair in beats.windows(2) {
        clock.set(pair[1]);
        lapses.push(has_lapsed(&clock, pair[0], Timedelta::from_secs(2)));
    }
    assert_eq!(lapses, [false, true], "the second gap, of three and a half seconds, lapsed");
}
// ANCHOR_END: replay

#[test]
// ANCHOR: threads
fn an_atomic_manual_clock_is_shared_across_threads() {
    use std::thread;

    use t2t::clock::{AtomicManualClock, Clock};
    use t2t::{Timedelta, Timestamp};

    let clock = AtomicManualClock::new(Timestamp::UNIX_EPOCH);
    thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for _ in 0..1_000 {
                    clock.advance(Timedelta::MICROSECOND);
                }
            });
        }
    });
    assert_eq!(clock.now(), Timestamp::from_millis(4), "every thread's moves count");
}
// ANCHOR_END: threads

#[test]
// ANCHOR: custom
fn a_test_writes_the_clock_it_needs() {
    use core::cell::Cell;

    use t2t::Timedelta;
    use t2t::clock::Clock;

    /// A CPU-time clock for a test: a span, which no manual clock reads, set by hand.
    #[derive(Debug, Default)]
    struct TestCpuClock(Cell<Timedelta>);

    impl Clock for TestCpuClock {
        type Reading = Timedelta;

        fn now(&self) -> Timedelta {
            self.0.get()
        }
    }

    let clock = TestCpuClock::default();
    clock.0.set(Timedelta::from_millis(3));
    assert_eq!(clock.now(), Timedelta::from_millis(3), "what the test set");
}
// ANCHOR_END: custom
