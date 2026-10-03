//! [`Timestamp`] and [`Timedelta`] to and from time's `OffsetDateTime` and `Duration`.

use time::{Duration, OffsetDateTime};

use crate::errors::narrow;
use crate::{OutOfRangeError, Timedelta, Timestamp};

/// Every timestamp, in UTC, since time's reach years -9999 to 9999.
impl From<Timestamp> for OffsetDateTime {
    #[inline]
    fn from(instant: Timestamp) -> Self {
        Self::UNIX_EPOCH.saturating_add(Duration::nanoseconds(instant.0))
    }
}

/// Refuses an instant outside [`Timestamp::MIN`] to [`Timestamp::MAX`], at any offset.
impl TryFrom<OffsetDateTime> for Timestamp {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(instant: OffsetDateTime) -> Result<Self, Self::Error> {
        narrow(instant.unix_timestamp_nanos()).map(Self)
    }
}

/// Every span, since a `Duration` holds every `i64` of nanoseconds.
impl From<Timedelta> for Duration {
    #[inline]
    fn from(span: Timedelta) -> Self {
        Self::nanoseconds(span.0)
    }
}

/// Refuses a duration past [`Timedelta::MIN`] or [`Timedelta::MAX`].
impl TryFrom<Duration> for Timedelta {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        narrow(duration.whole_nanoseconds()).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use time::{Duration, OffsetDateTime, UtcOffset};

    use crate::{OutOfRangeError, Timedelta, Timestamp};

    #[rstest]
    #[case::the_earliest(Timestamp::MIN)]
    #[case::before_the_epoch(Timestamp::from_nanos(-1))]
    #[case::a_capture(Timestamp::from_nanos(1_789_544_735_123_456_789))]
    #[case::the_latest(Timestamp::MAX)]
    fn an_instant_crosses_both_ways(#[case] instant: Timestamp) {
        let theirs = OffsetDateTime::from(instant);
        assert_eq!(
            theirs.unix_timestamp_nanos(),
            i128::from(instant.as_nanos()),
            "the same instant"
        );
        assert_eq!(theirs.offset(), UtcOffset::UTC, "in UTC");
        assert_eq!(Timestamp::try_from(theirs), Ok(instant), "and back");
    }

    #[test]
    fn an_instant_at_another_offset_is_the_same_instant() {
        let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
        let offset = UtcOffset::from_hms(5, 0, 0).expect("five hours east is an offset");
        let elsewhere = OffsetDateTime::from(instant).to_offset(offset);
        assert_eq!(Timestamp::try_from(elsewhere), Ok(instant), "read in UTC");
    }

    #[test]
    fn an_instant_past_the_range_is_refused() {
        let far_future = OffsetDateTime::UNIX_EPOCH.saturating_add(Duration::MAX);
        let distant_past = OffsetDateTime::UNIX_EPOCH.saturating_add(Duration::MIN);
        assert_eq!(Timestamp::try_from(far_future), Err(OutOfRangeError), "after 2262");
        assert_eq!(Timestamp::try_from(distant_past), Err(OutOfRangeError), "before 1677");
    }

    #[rstest]
    #[case::the_longest_backwards(Timedelta::MIN)]
    #[case::a_budget(Timedelta::from_millis(-250))]
    #[case::the_longest(Timedelta::MAX)]
    fn a_span_crosses_both_ways(#[case] span: Timedelta) {
        let theirs = Duration::from(span);
        assert_eq!(theirs.whole_nanoseconds(), i128::from(span.as_nanos()), "the same span");
        assert_eq!(Timedelta::try_from(theirs), Ok(span), "and back");
    }

    #[test]
    fn a_span_past_the_range_is_refused() {
        assert_eq!(Timedelta::try_from(Duration::MAX), Err(OutOfRangeError), "forwards");
        assert_eq!(Timedelta::try_from(Duration::MIN), Err(OutOfRangeError), "and backwards");
    }
}
