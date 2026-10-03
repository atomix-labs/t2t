//! The listings of the chapter Dates and Spellings.

#[test]
// ANCHOR: calendar
fn a_timestamp_reads_as_a_date_and_back() {
    use t2t::{Timestamp, UtcDateTime};

    let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
    let date_time = instant.to_utc();
    assert_eq!((date_time.year, date_time.month, date_time.day), (2026, 9, 16), "the date");
    assert_eq!((date_time.hour, date_time.minute, date_time.second), (7, 45, 35), "the time");
    assert_eq!(date_time.nanosecond, 123_456_789, "to the nanosecond");
    assert_eq!(date_time.to_timestamp(), Some(instant), "and back to the instant");

    // The fields are public, so a date and time can be built by hand, and checked.
    let leap_day = UtcDateTime { year: 2028, month: 2, day: 29, ..date_time };
    assert!(leap_day.is_valid(), "2028 is a leap year");
    let no_such_day = UtcDateTime { day: 30, ..leap_day };
    assert!(!no_such_day.is_valid(), "February has no 30th");
    assert_eq!(no_such_day.to_timestamp(), None, "so it names no instant");
    let too_late = UtcDateTime { year: 2263, ..date_time };
    assert_eq!(too_late.to_timestamp(), None, "nor does a date past 2262");
}
// ANCHOR_END: calendar

#[test]
// ANCHOR: write
fn an_instant_is_written_as_rfc_3339() {
    use t2t::Timestamp;

    let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
    assert_eq!(instant.to_string(), "2026-09-16T07:45:35.123456789Z", "nine digits");
    assert_eq!(format!("{instant:.3}"), "2026-09-16T07:45:35.123Z", "to the millisecond");
    assert_eq!(format!("{instant:.0}"), "2026-09-16T07:45:35Z", "to the second");
    assert_eq!(format!("{instant:.12}"), instant.to_string(), "never past the nanosecond");
    assert_eq!(format!("[{instant:>24.0}]"), "[    2026-09-16T07:45:35Z]", "padded to a width");
    assert_eq!(format!("{instant:?}"), instant.to_string(), "and debugged as written");
}
// ANCHOR_END: write

#[test]
// ANCHOR: read
fn an_instant_is_read_from_rfc_3339_in_utc() {
    use t2t::{ParseTimestampError, Timestamp};

    let read = "2026-09-16T07:45:35.5Z".parse::<Timestamp>();
    assert_eq!(read, Ok(Timestamp::from_millis(1_789_544_735_500)), "a fraction of any length");
    let lower = "1970-01-01t00:00:00z".parse::<Timestamp>();
    assert_eq!(lower, Ok(Timestamp::UNIX_EPOCH), "and either case");

    for (refused, reason) in [
        ("2026-09-16T09:45:35+02:00", "an offset: an instant is read in UTC alone"),
        ("2026-09-16T07:45:60Z", "a leap second, which Unix time has none of"),
        ("2026-02-30T07:45:35Z", "a date that does not exist"),
        ("2263-01-01T00:00:00Z", "a date past 2262"),
        ("2026-09-16T07:45:35.1234567890Z", "a tenth fraction digit"),
        ("2026-09-16 07:45:35Z", "a space for the `T`"),
    ] {
        assert_eq!(refused.parse::<Timestamp>(), Err(ParseTimestampError), "{reason} is refused");
    }
}
// ANCHOR_END: read

#[test]
// ANCHOR: tai
fn a_tai_instant_is_written_in_its_own_zone() {
    use t2t::{ParseTaiTimestampError, TaiTimestamp, Timedelta};

    let synced: TaiTimestamp = "2026-09-16T07:46:12.5 TAI".parse().expect("an instant in TAI");
    let next = synced + Timedelta::from_millis(500);
    assert_eq!(next.to_string(), "2026-09-16T07:46:13.000000000 TAI", "written in TAI");
    assert_eq!(format!("{next:.0}"), "2026-09-16T07:46:13 TAI", "to the precision asked");

    let utc = "2026-09-16T07:46:12Z".parse::<TaiTimestamp>();
    assert_eq!(utc, Err(ParseTaiTimestampError), "an instant in UTC is not one in TAI");
}
// ANCHOR_END: tai

#[test]
// ANCHOR: spans
fn a_span_is_spelled_coarsest_unit_first() {
    use t2t::{ParseTimedeltaError, Timedelta, Uptime};

    assert_eq!(Timedelta::from_secs(90).to_string(), "1m30s", "coarsest first");
    assert_eq!(Timedelta::from_millis(60_123).to_string(), "1m0s123ms", "a zero inside is written");
    assert_eq!(Timedelta::from_micros(-3).to_string(), "-3us", "a sign for a span backwards");
    assert_eq!(Timedelta::ZERO.to_string(), "0s", "and zero in seconds");
    assert_eq!(Uptime::from_millis(42_050).to_string(), "42s50ms", "a point from boot alike");

    assert_eq!("48h".parse(), Ok(Timedelta::from_days(2)), "a count may pass the next unit");
    assert_eq!("0".parse(), Ok(Timedelta::ZERO), "and zero may be bare");
    for refused in ["90 seconds", "1s1m", "1m1m", "+1s", "-0", "1"] {
        assert_eq!(refused.parse::<Timedelta>(), Err(ParseTimedeltaError), "{refused} refused");
    }
}
// ANCHOR_END: spans

#[test]
// ANCHOR: counts
fn a_count_is_spelled_with_its_unit() {
    use t2t::{ParseTickRateError, ParseTickdeltaError, TickRate, Tickdelta, Tickstamp};

    assert_eq!(Tickdelta::from_ticks(-24).to_string(), "-24 ticks", "a span of ticks");
    assert_eq!(Tickstamp::from_ticks(1_000).to_string(), "1000 ticks", "a counter's reading");
    assert_eq!(TickRate::GIGAHERTZ.to_string(), "1000000000 Hz", "and a rate");
    assert_eq!(format!("[{:>10}]", Tickdelta::from_ticks(24)), "[  24 ticks]", "padded whole");

    assert_eq!("24 ticks".parse(), Ok(Tickdelta::from_ticks(24)), "read back");
    assert_eq!("24".parse::<Tickdelta>(), Err(ParseTickdeltaError), "a count names its unit");
    assert_eq!("0 Hz".parse::<TickRate>(), Err(ParseTickRateError), "and a rate is above zero");
}
// ANCHOR_END: counts

#[test]
// ANCHOR: refusals
fn values_that_share_a_spelling_share_its_refusal() {
    use t2t::{BootUptime, ParseTickdeltaError, ParseTimedeltaError, Tickstamp, Uptime};

    assert_eq!(
        "soon".parse::<Uptime>(),
        Err(ParseTimedeltaError),
        "an uptime is a span's spelling"
    );
    assert_eq!("soon".parse::<BootUptime>(), Err(ParseTimedeltaError), "and so is a boot uptime");
    assert_eq!("soon".parse::<Tickstamp>(), Err(ParseTickdeltaError), "a reading is a count's");
    assert_eq!(
        ParseTimedeltaError.to_string(),
        "parse timedelta error: expected `0`, or counts with units from coarsest to finest, as \
         `1m30s`",
        "each refusal names the spelling it expected"
    );
}
// ANCHOR_END: refusals
