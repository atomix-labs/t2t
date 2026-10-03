//! The listings of the chapter Working with Other Crates, each built where its
//! crate's feature is on.

#[test]
// ANCHOR: duration
fn a_span_crosses_to_a_duration_where_it_runs_forwards() {
    use core::time::Duration;

    use t2t_core::{OutOfRangeError, Timedelta};

    let span = Timedelta::from_millis(5);
    assert_eq!(Duration::try_from(span), Ok(Duration::from_millis(5)), "a span forwards");
    assert_eq!(Timedelta::try_from(Duration::from_millis(5)), Ok(span), "and back");
    assert_eq!(Duration::try_from(-span), Err(OutOfRangeError), "no duration runs backwards");
    assert_eq!(Timedelta::try_from(Duration::MAX), Err(OutOfRangeError), "nor past 292 years");
}
// ANCHOR_END: duration

#[test]
#[cfg(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
))]
// ANCHOR: system-time
fn a_timestamp_crosses_to_a_system_time_and_back_within_its_range() {
    use core::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    use t2t_core::{OutOfRangeError, Timestamp};

    let instant = Timestamp::from_secs(1_700_000_000);
    let system_time = SystemTime::from(instant);
    let since_epoch = system_time.duration_since(UNIX_EPOCH).ok();
    assert_eq!(since_epoch, Some(Duration::from_secs(1_700_000_000)), "the same moment");
    assert_eq!(Timestamp::try_from(system_time), Ok(instant), "and back");

    let far_future = UNIX_EPOCH
        .checked_add(Duration::from_secs(1 << 40))
        .expect("a 64-bit SystemTime reaches 2^40 seconds past 1970");
    assert_eq!(Timestamp::try_from(far_future), Err(OutOfRangeError), "past 2262 is refused");
}
// ANCHOR_END: system-time

#[test]
#[cfg(feature = "chrono-04")]
// ANCHOR: chrono
fn a_timestamp_and_a_span_cross_to_chrono() {
    use chrono::{DateTime, FixedOffset, TimeDelta, Utc};
    use t2t_core::{OutOfRangeError, Timedelta, Timestamp};

    let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
    let theirs = DateTime::<Utc>::from(instant);
    assert_eq!(theirs.timestamp_nanos_opt(), Some(instant.as_nanos()), "the same instant");

    let paris = theirs.with_timezone(&FixedOffset::east_opt(7_200).expect("two hours east"));
    assert_eq!(Timestamp::try_from(paris), Ok(instant), "in any zone, read as its instant");
    let latest = DateTime::<Utc>::MAX_UTC;
    assert_eq!(Timestamp::try_from(latest), Err(OutOfRangeError), "past 2262 is refused");

    let span = Timedelta::from_millis(-250);
    assert_eq!(TimeDelta::from(span), TimeDelta::milliseconds(-250), "every span crosses");
    assert_eq!(Timedelta::try_from(TimeDelta::MAX), Err(OutOfRangeError), "but not every back");
}
// ANCHOR_END: chrono

#[test]
#[cfg(feature = "jiff-02")]
// ANCHOR: jiff
fn a_timestamp_and_a_span_cross_to_jiff() {
    use jiff::{SignedDuration, Timestamp as JiffTimestamp};
    use t2t_core::{OutOfRangeError, Timedelta, Timestamp};

    let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
    let theirs = JiffTimestamp::from(instant);
    assert_eq!(theirs.to_string(), "2026-09-16T07:45:35.123456789Z", "the same instant");
    assert_eq!(Timestamp::try_from(theirs), Ok(instant), "and back");
    let latest = JiffTimestamp::MAX;
    assert_eq!(Timestamp::try_from(latest), Err(OutOfRangeError), "past 2262 is refused");

    let span = Timedelta::from_millis(-250);
    assert_eq!(SignedDuration::from(span), SignedDuration::from_millis(-250), "a span");
    let longest = SignedDuration::MAX;
    assert_eq!(Timedelta::try_from(longest), Err(OutOfRangeError), "and one too long for it");
}
// ANCHOR_END: jiff

#[test]
#[cfg(feature = "time-03")]
// ANCHOR: time
fn a_timestamp_and_a_span_cross_to_time() {
    use t2t_core::{OutOfRangeError, Timedelta, Timestamp};
    use time::{Duration, OffsetDateTime, UtcOffset};

    let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
    let theirs = OffsetDateTime::from(instant);
    assert_eq!(theirs.offset(), UtcOffset::UTC, "in UTC");
    let east = theirs.to_offset(UtcOffset::from_hms(2, 0, 0).expect("two hours east"));
    assert_eq!(Timestamp::try_from(east), Ok(instant), "at any offset, read as its instant");

    let span = Timedelta::from_millis(-250);
    assert_eq!(Duration::from(span), Duration::milliseconds(-250), "a span");
    assert_eq!(Timedelta::try_from(Duration::MAX), Err(OutOfRangeError), "and one too long");
}
// ANCHOR_END: time
