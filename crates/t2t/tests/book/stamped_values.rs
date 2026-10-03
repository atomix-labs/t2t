//! The listings of the chapter Stamped Values.

#[test]
// ANCHOR: timed
fn a_stamped_value_says_how_old_it_is() {
    use t2t::{Timed, Timedelta, Timestamp};

    /// The best prices on a book, in the instrument's price ticks.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Quote {
        /// The best bid.
        bid: u64,
        /// The best offer.
        ask: u64,
    }

    let captured = Timestamp::from_secs(1_700_000_000);
    let quote = Timed::new(captured, Quote { bid: 100, ask: 101 });

    assert_eq!(quote.bid, 100, "the value's fields, through `Deref`");
    let now = captured + Timedelta::from_millis(1_500);
    assert_eq!(quote.elapsed(now), Timedelta::from_millis(1_500), "its age at `now`");
    assert!(quote.elapsed(now) > Timedelta::SECOND, "so it is stale after a second");
    let early = captured - Timedelta::MILLISECOND;
    assert!(quote.elapsed(early).is_negative(), "and its age is negative before its stamp");
}
// ANCHOR_END: timed

#[test]
// ANCHOR: writer
fn the_writer_stamps_and_the_reader_keeps_the_stamp() {
    use t2t::clock::{Clock, ManualClock};
    use t2t::{Timed, Timedelta, Timestamp};

    /// A value, stamped by `clock` as it is written.
    fn publish<C: Clock<Reading = Timestamp>, T>(clock: &C, value: T) -> Timed<T> {
        Timed::new(clock.now(), value)
    }

    let clock = ManualClock::new(Timestamp::from_secs(1_700_000_000));
    let price = publish(&clock, 101_u64);
    clock.advance(Timedelta::from_millis(3));

    // Whatever the reader makes of the value, the stamp stays the writer's.
    let doubled = price.map(|ticks| ticks.saturating_mul(2));
    assert_eq!((doubled.stamp, doubled.value), (price.stamp, 202), "a new value, the same stamp");
    let in_seconds = price.map_stamp(Timestamp::as_secs);
    assert_eq!(in_seconds.stamp, 1_700_000_000, "or a new stamp over the same value");
    assert_eq!(price.elapsed(clock.now()), Timedelta::from_millis(3), "and its age, read late");
    assert_eq!(price.into_parts(), (price.stamp, 101), "and its parts, the stamp first");
}
// ANCHOR_END: writer

#[test]
// ANCHOR: order
fn stamped_values_sort_by_their_stamps() {
    use t2t::{Timed, Uptime};

    let mut events = [
        Timed::new(Uptime::from_secs(3), "third"),
        Timed::new(Uptime::from_secs(1), "first"),
        Timed::new(Uptime::from_secs(2), "second"),
    ];
    events.sort();
    let order = events.map(Timed::into_inner);
    assert_eq!(order, ["first", "second", "third"], "the stamp decides the order");
}
// ANCHOR_END: order

#[test]
// ANCHOR: layout
fn a_stamped_word_is_two_words() {
    use core::mem::offset_of;

    use t2t::{Tickstamp, Timed};

    assert_eq!(size_of::<Timed<u64>>(), 16, "a stamp and a word, no padding");
    assert_eq!(offset_of!(Timed<u64>, stamp), 0, "the stamp first");
    assert_eq!(size_of::<Timed<u64, Tickstamp>>(), 16, "on any timeline");
}
// ANCHOR_END: layout
