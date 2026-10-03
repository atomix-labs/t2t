//! [`Timestamp`] and [`Timedelta`] to and from chrono's `DateTime` and `TimeDelta`.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};

use crate::{OutOfRangeError, Timedelta, Timestamp};

/// Every timestamp, in UTC, since chrono's reach years -262143 to 262142.
impl From<Timestamp> for DateTime<Utc> {
    #[inline]
    fn from(instant: Timestamp) -> Self {
        Self::from_timestamp_nanos(instant.0)
    }
}

/// Refuses an instant outside [`Timestamp::MIN`] to [`Timestamp::MAX`], in any zone.
impl<Z: TimeZone> TryFrom<DateTime<Z>> for Timestamp {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(instant: DateTime<Z>) -> Result<Self, Self::Error> {
        instant.timestamp_nanos_opt().map(Self).ok_or(OutOfRangeError)
    }
}

/// Every span, since a `TimeDelta` holds every `i64` of nanoseconds.
impl From<Timedelta> for TimeDelta {
    #[inline]
    fn from(span: Timedelta) -> Self {
        Self::nanoseconds(span.0)
    }
}

/// Refuses a duration past [`Timedelta::MIN`] or [`Timedelta::MAX`].
impl TryFrom<TimeDelta> for Timedelta {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(span: TimeDelta) -> Result<Self, Self::Error> {
        span.num_nanoseconds().map(Self).ok_or(OutOfRangeError)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset, TimeDelta, Utc};
    use rstest::rstest;

    use crate::{OutOfRangeError, Timedelta, Timestamp};

    #[rstest]
    #[case::the_earliest(Timestamp::MIN)]
    #[case::before_the_epoch(Timestamp::from_nanos(-1))]
    #[case::a_capture(Timestamp::from_nanos(1_789_544_735_123_456_789))]
    #[case::the_latest(Timestamp::MAX)]
    fn an_instant_crosses_both_ways(#[case] instant: Timestamp) {
        let theirs = DateTime::<Utc>::from(instant);
        assert_eq!(theirs.timestamp_nanos_opt(), Some(instant.as_nanos()), "the same instant");
        assert_eq!(Timestamp::try_from(theirs), Ok(instant), "and back");
    }

    #[test]
    fn an_instant_in_another_zone_is_the_same_instant() {
        let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
        let zone = FixedOffset::east_opt(18_000).expect("five hours east is an offset");
        let elsewhere = DateTime::<Utc>::from(instant).with_timezone(&zone);
        assert_eq!(Timestamp::try_from(elsewhere), Ok(instant), "read in UTC");
    }

    #[test]
    fn an_instant_past_the_range_is_refused() {
        assert_eq!(
            Timestamp::try_from(DateTime::<Utc>::MAX_UTC),
            Err(OutOfRangeError),
            "after 2262"
        );
        assert_eq!(
            Timestamp::try_from(DateTime::<Utc>::MIN_UTC),
            Err(OutOfRangeError),
            "before 1677"
        );
    }

    #[rstest]
    #[case::the_longest_backwards(Timedelta::MIN)]
    #[case::a_budget(Timedelta::from_millis(-250))]
    #[case::the_longest(Timedelta::MAX)]
    fn a_span_crosses_both_ways(#[case] span: Timedelta) {
        let theirs = TimeDelta::from(span);
        assert_eq!(theirs.num_nanoseconds(), Some(span.as_nanos()), "the same span");
        assert_eq!(Timedelta::try_from(theirs), Ok(span), "and back");
    }

    #[test]
    fn a_span_past_the_range_is_refused() {
        assert_eq!(Timedelta::try_from(TimeDelta::MAX), Err(OutOfRangeError), "forwards");
        assert_eq!(Timedelta::try_from(TimeDelta::MIN), Err(OutOfRangeError), "and backwards");
    }
}
