//! The listings of the chapter Points and Spans.

#[test]
// ANCHOR: arithmetic
fn a_point_minus_a_point_is_a_span() {
    use t2t::{Timedelta, Timestamp};

    let published = Timestamp::from_secs(1_700_000_000);
    let captured = published + Timedelta::from_micros(850);
    let latency = captured - published;

    assert_eq!(latency, Timedelta::from_micros(850), "a point minus a point is a span");
    assert_eq!(captured - latency, published, "and a point minus a span is a point");
    assert_eq!(latency * 3, Timedelta::from_micros(2_550), "a span scales by a count");
    assert!(latency < Timedelta::MILLISECOND, "and compares with another span");
}
// ANCHOR_END: arithmetic

#[test]
// ANCHOR: units
fn each_unit_has_a_constructor_and_an_accessor() {
    use t2t::{Timedelta, Timestamp, Uptime};

    assert_eq!(Timedelta::from_mins(90), Timedelta::HOUR + Timedelta::from_mins(30), "a span");
    assert_eq!(Uptime::from_secs(90).as_millis(), 90_000, "a point, counted in a finer unit");

    // A span counts whole units toward zero, as std's `Duration` does.
    assert_eq!(Timedelta::from_millis(1_500).as_secs(), 1, "forwards");
    assert_eq!(Timedelta::from_millis(-1_500).as_secs(), -1, "and backwards");
    assert_eq!(Timedelta::from_millis(-1_500).subsec_millis(), -500, "the rest has its sign");

    // A point counts the whole units since its origin, so one before it rounds down.
    assert_eq!(Timestamp::from_millis(-1_500).as_secs(), -2, "the second it falls in");
}
// ANCHOR_END: units

#[test]
// ANCHOR: saturating
fn an_operator_saturates_and_its_checked_twin_refuses() {
    use t2t::{Timedelta, Timestamp};

    let latest = Timestamp::MAX;
    assert_eq!(latest + Timedelta::SECOND, Timestamp::MAX, "the sum stops at the end");
    assert_eq!(latest.checked_add(Timedelta::SECOND), None, "where its twin refuses");

    let widest = Timestamp::MAX - Timestamp::MIN;
    assert_eq!(widest, Timedelta::MAX, "a span past its range stops at its end too");
    assert_eq!(Timestamp::MAX.checked_since(Timestamp::MIN), None, "or is refused");

    assert_eq!(Timedelta::from_secs(i64::MAX), Timedelta::MAX, "a constructor saturates");
    assert_eq!(-Timedelta::MIN, Timedelta::MAX, "and so does a negation");
    assert_eq!(Timedelta::MIN.checked_neg(), None, "which its twin refuses");

    // A span has no `/`; `checked_div` divides it, and refuses zero parts.
    let third = Timedelta::SECOND.checked_div(3);
    assert_eq!(third, Some(Timedelta::from_nanos(333_333_333)), "truncated toward zero");
    assert_eq!(Timedelta::SECOND.checked_div(0), None, "and refused for no parts");
}
// ANCHOR_END: saturating

#[test]
// ANCHOR: floor
fn floor_and_ceil_find_the_bucket_a_point_falls_in() {
    use t2t::{Timedelta, Timestamp};

    let event: Timestamp = "2026-09-16T07:45:35.123Z".parse().expect("an instant in UTC");
    let start = event.floor(Timedelta::MINUTE);
    let end = event.ceil(Timedelta::MINUTE);

    assert_eq!(format!("{start:.0}"), "2026-09-16T07:45:00Z", "the minute it falls in");
    assert_eq!(format!("{end:.0}"), "2026-09-16T07:46:00Z", "and the next");
    assert_eq!(start.floor(Timedelta::MINUTE), start, "a multiple stays where it is");
    assert_eq!(event.floor(Timedelta::ZERO), event, "and a zero unit leaves the point");

    let budget = Timedelta::from_micros(1_250);
    assert_eq!(budget.ceil(Timedelta::MILLISECOND), Timedelta::from_millis(2), "a span too");
}
// ANCHOR_END: floor

#[test]
// ANCHOR: generic
fn code_generic_over_points_takes_every_timeline() {
    use t2t::{Tickdelta, Tickstamp, TimePoint, Timedelta, Uptime};

    /// How long ago `stamp` is, at `now`, on whichever timeline the two are on.
    fn age<P: TimePoint>(stamp: P, now: P) -> P::Span {
        now - stamp
    }

    let span = age(Uptime::from_secs(5), Uptime::from_secs(7));
    assert_eq!(span, Timedelta::from_secs(2), "a span of nanoseconds on the monotonic clock");
    let ticks = age(Tickstamp::from_ticks(100), Tickstamp::from_ticks(124));
    assert_eq!(ticks, Tickdelta::from_ticks(24), "and a span of ticks on a counter");
    assert_eq!(Uptime::from_count(span.as_nanos()), Uptime::from_secs(2), "from a count");
}
// ANCHOR_END: generic
