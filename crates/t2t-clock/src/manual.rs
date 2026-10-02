//! Clocks set and moved by hand, for tests and replays.

use core::cell::Cell;
use core::fmt;
#[cfg(target_has_atomic = "64")]
use core::marker::PhantomData;

use t2t_core::{TimePoint, Timestamp};

use crate::Clock;
#[cfg(target_has_atomic = "64")]
use crate::sync::{AtomicI64, Ordering};

/// A clock set and moved by hand, read on one thread.
///
/// Reading it costs a load; [`AtomicManualClock`] is the one to share across threads. It is not
/// `Clone`, since a copy would fork time: share it by reference.
///
/// # Examples
/// ```
/// use t2t_clock::{Clock, ManualClock};
/// use t2t_core::{Timedelta, Timestamp};
///
/// let clock = ManualClock::new(Timestamp::UNIX_EPOCH);
/// clock.advance(Timedelta::SECOND);
/// assert_eq!(clock.now(), Timestamp::from_secs(1), "moved on by a second");
/// clock.set(Timestamp::from_secs(60));
/// assert_eq!(clock.now(), Timestamp::from_secs(60), "moved to a minute");
/// ```
pub struct ManualClock<P = Timestamp> {
    /// The point the clock reads.
    reading: Cell<P>,
}

impl<P: TimePoint> ManualClock<P> {
    /// A clock reading `start`.
    #[must_use]
    pub const fn new(start: P) -> Self {
        Self { reading: Cell::new(start) }
    }

    /// Moves the clock to `point`, earlier or later.
    #[inline]
    pub fn set(&self, point: P) {
        self.reading.set(point);
    }

    /// Moves the clock by `span`, back for a negative one, saturating as the point's addition does.
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "the point's own addition, which saturates for t2t's"
    )]
    pub fn advance(&self, span: P::Span) {
        self.reading.set(self.reading.get() + span);
    }
}

impl<P: TimePoint> Clock for ManualClock<P> {
    type Instant = P;

    #[inline]
    fn now(&self) -> P {
        self.reading.get()
    }
}

impl<P: TimePoint + Default> Default for ManualClock<P> {
    fn default() -> Self {
        Self::new(P::default())
    }
}

impl<P: TimePoint + fmt::Debug> fmt::Debug for ManualClock<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("ManualClock").field(&self.reading.get()).finish()
    }
}

/// A clock set and moved by hand, shared across threads.
///
/// A thread that reads a point another set or moved the clock to also sees what that thread wrote
/// before. It is not `Clone`, since a copy would fork time: share it by reference or `Arc`.
///
/// # Examples
/// ```
/// use std::thread;
///
/// use t2t_clock::{AtomicManualClock, Clock};
/// use t2t_core::{Timedelta, Uptime};
///
/// let clock = AtomicManualClock::new(Uptime::from_secs(1));
/// thread::scope(|scope| {
///     scope.spawn(|| clock.advance(Timedelta::SECOND));
/// });
/// assert_eq!(clock.now(), Uptime::from_secs(2), "the other thread's move");
/// ```
#[cfg(target_has_atomic = "64")]
pub struct AtomicManualClock<P = Timestamp> {
    /// The count of the point the clock reads.
    reading: AtomicI64,
    /// The kind of point, held by no value, so the clock is `Send` and `Sync` whatever it is.
    marker: PhantomData<fn() -> P>,
}

#[cfg(target_has_atomic = "64")]
impl<P: TimePoint> AtomicManualClock<P> {
    /// A clock reading `start`.
    #[must_use]
    pub fn new(start: P) -> Self {
        Self { reading: AtomicI64::new(start.to_i64()), marker: PhantomData }
    }

    /// Moves the clock to `point`, earlier or later.
    #[inline]
    pub fn set(&self, point: P) {
        // ORDERING: Release, pairing with the Acquire load in `now`.
        self.reading.store(point.to_i64(), Ordering::Release);
    }

    /// Moves the clock by `span`, back for a negative one, saturating as the point's addition does.
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "the point's own addition, which saturates for t2t's"
    )]
    pub fn advance(&self, span: P::Span) {
        // ORDERING: Relaxed; the count only seeds the exchange, which orders the move.
        let mut expected = self.reading.load(Ordering::Relaxed);
        loop {
            let target = (P::from_i64(expected) + span).to_i64();
            // ORDERING: AcqRel on success, pairing with the Acquire load in `now` and with the
            // Release of the last `set` or `advance`; Relaxed on failure, which only reloads.
            match self.reading.compare_exchange_weak(
                expected,
                target,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => expected = actual,
            }
        }
    }
}

// What the docs promise of a clock to share.
#[cfg(target_has_atomic = "64")]
const _: () = {
    /// Compiles only for a type that is `Send` and `Sync`.
    const fn is_send_and_sync<T: Send + Sync>() {}
    is_send_and_sync::<AtomicManualClock<Timestamp>>();
};

#[cfg(target_has_atomic = "64")]
impl<P: TimePoint> Clock for AtomicManualClock<P> {
    type Instant = P;

    #[inline]
    fn now(&self) -> P {
        // ORDERING: Acquire, pairing with the Release of whichever thread set or moved the clock.
        P::from_i64(self.reading.load(Ordering::Acquire))
    }
}

#[cfg(target_has_atomic = "64")]
impl<P: TimePoint + Default> Default for AtomicManualClock<P> {
    fn default() -> Self {
        Self::new(P::default())
    }
}

#[cfg(target_has_atomic = "64")]
impl<P: TimePoint + fmt::Debug> fmt::Debug for AtomicManualClock<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("AtomicManualClock").field(&self.now()).finish()
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use std::thread;

    use t2t_core::{Tick, Ticks, Timedelta, Timestamp};

    use crate::{AtomicManualClock, Clock, ManualClock};

    #[test]
    fn a_manual_clock_moves_both_ways_and_saturates() {
        let clock = ManualClock::new(Tick::new(100));
        clock.advance(Ticks::new(-40));
        assert_eq!(clock.now(), Tick::new(60));
        clock.set(Tick::MAX);
        clock.advance(Ticks::new(1));
        assert_eq!(clock.now(), Tick::MAX);
    }

    #[test]
    fn an_atomic_manual_clock_sums_every_threads_moves() {
        let clock = AtomicManualClock::new(Timestamp::UNIX_EPOCH);
        thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| {
                    for _ in 0..1_000 {
                        clock.advance(Timedelta::NANOSECOND);
                    }
                });
            }
        });
        assert_eq!(clock.now(), Timestamp::from_nanos(8_000));
    }

    #[test]
    fn a_clock_is_read_through_a_reference() {
        let clock = ManualClock::<Timestamp>::default();
        let reference: &dyn Clock<Instant = Timestamp> = &clock;
        assert_eq!(reference.now(), Timestamp::UNIX_EPOCH);
        assert_eq!(std::format!("{clock:?}"), "ManualClock(1970-01-01T00:00:00.000000000Z)");
    }
}

// The atomic clock under loom, which runs each model once for every interleaving of its two threads
// that the memory model allows: `cargo test -p t2t-clock --lib --release --config
// 'target."cfg(all())".rustflags=["--cfg","loom"]'`. Loom has two gaps here: it orders a store and
// an exchange that did not see it only partly, so a `set` racing an `advance` is not modelled; and
// past `advance`'s seed load, its search misses an exchange without Release.
#[cfg(test)]
#[cfg(loom)]
mod model {
    use loom::sync::Arc;
    use loom::sync::atomic::{AtomicU64, Ordering};
    use loom::thread;
    use t2t_core::{Timedelta, Timestamp};

    use crate::{AtomicManualClock, Clock};

    /// A clock at the epoch, and a note for a thread to write before it moves the clock.
    fn clock_and_note() -> (Arc<AtomicManualClock>, Arc<AtomicU64>) {
        (Arc::new(AtomicManualClock::new(Timestamp::UNIX_EPOCH)), Arc::new(AtomicU64::new(0)))
    }

    #[test]
    fn a_reader_that_sees_a_set_point_sees_what_was_written_before_it() {
        loom::model(|| {
            let (clock, note) = clock_and_note();
            let setter = {
                let (clock, note) = (Arc::clone(&clock), Arc::clone(&note));
                thread::spawn(move || {
                    // ORDERING: Relaxed; the Release in `set` publishes it.
                    note.store(7, Ordering::Relaxed);
                    clock.set(Timestamp::from_secs(1));
                })
            };
            if clock.now() == Timestamp::from_secs(1) {
                // ORDERING: Relaxed; the Acquire in `now`, which saw the set, orders it after.
                assert_eq!(note.load(Ordering::Relaxed), 7, "the note written before the set");
            }
            setter.join().expect("the setting thread ends");
        });
    }

    #[test]
    fn a_reader_that_sees_an_advance_sees_what_was_written_before_it() {
        loom::model(|| {
            let (clock, note) = clock_and_note();
            let advancer = {
                let (clock, note) = (Arc::clone(&clock), Arc::clone(&note));
                thread::spawn(move || {
                    // ORDERING: Relaxed; the AcqRel exchange in `advance` publishes it.
                    note.store(7, Ordering::Relaxed);
                    clock.advance(Timedelta::SECOND);
                })
            };
            if clock.now() == Timestamp::from_secs(1) {
                // ORDERING: Relaxed; the Acquire in `now`, which saw the advance, orders it after.
                assert_eq!(note.load(Ordering::Relaxed), 7, "the note written before the advance");
            }
            advancer.join().expect("the advancing thread ends");
        });
    }

    #[test]
    fn two_threads_advancing_at_once_both_count() {
        loom::model(|| {
            let (clock, _) = clock_and_note();
            let advancer = {
                let clock = Arc::clone(&clock);
                thread::spawn(move || clock.advance(Timedelta::SECOND))
            };
            clock.advance(Timedelta::SECOND);
            advancer.join().expect("the advancing thread ends");
            assert_eq!(clock.now(), Timestamp::from_secs(2), "neither move lost");
        });
    }
}
