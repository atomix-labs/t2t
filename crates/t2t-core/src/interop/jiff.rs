//! [`Timestamp`] and [`Timedelta`] to and from jiff's `Timestamp` and `SignedDuration`.

use jiff::{SignedDuration, Timestamp as JiffTimestamp};

use crate::{OutOfRangeError, Timedelta, Timestamp};

/// Every timestamp, since jiff's reach years -9999 to 9999.
impl From<Timestamp> for JiffTimestamp {
    #[inline]
    fn from(instant: Timestamp) -> Self {
        // jiff refuses only an instant past its years, which no timestamp reaches; were one to,
        // the conversion would saturate, as t2t's operators do.
        let saturated = if instant.0 < 0 { Self::MIN } else { Self::MAX };
        Self::from_nanosecond(i128::from(instant.0)).unwrap_or(saturated)
    }
}

/// Refuses an instant outside [`Timestamp::MIN`] to [`Timestamp::MAX`].
impl TryFrom<JiffTimestamp> for Timestamp {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(instant: JiffTimestamp) -> Result<Self, Self::Error> {
        i64::try_from(instant.as_nanosecond()).map(Self).map_err(|_past_the_range| OutOfRangeError)
    }
}

/// Every span, since a `SignedDuration` holds every `i64` of nanoseconds.
impl From<Timedelta> for SignedDuration {
    #[inline]
    fn from(span: Timedelta) -> Self {
        Self::from_nanos(span.0)
    }
}

/// Refuses a duration past [`Timedelta::MIN`] or [`Timedelta::MAX`].
impl TryFrom<SignedDuration> for Timedelta {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(duration: SignedDuration) -> Result<Self, Self::Error> {
        i64::try_from(duration.as_nanos()).map(Self).map_err(|_past_the_range| OutOfRangeError)
    }
}

#[cfg(test)]
mod tests {
    use jiff::{SignedDuration, Timestamp as JiffTimestamp};
    use rstest::rstest;

    use crate::{OutOfRangeError, Timedelta, Timestamp};

    #[rstest]
    #[case::the_earliest(Timestamp::MIN)]
    #[case::before_the_epoch(Timestamp::from_nanos(-1))]
    #[case::a_capture(Timestamp::from_nanos(1_789_544_735_123_456_789))]
    #[case::the_latest(Timestamp::MAX)]
    fn an_instant_crosses_both_ways(#[case] instant: Timestamp) {
        let theirs = JiffTimestamp::from(instant);
        assert_eq!(theirs.as_nanosecond(), i128::from(instant.as_nanos()), "the same instant");
        assert_eq!(Timestamp::try_from(theirs), Ok(instant), "and back");
    }

    #[test]
    fn an_instant_past_the_range_is_refused() {
        assert_eq!(Timestamp::try_from(JiffTimestamp::MAX), Err(OutOfRangeError), "after 2262");
        assert_eq!(Timestamp::try_from(JiffTimestamp::MIN), Err(OutOfRangeError), "before 1677");
    }

    #[rstest]
    #[case::the_longest_backwards(Timedelta::MIN)]
    #[case::a_budget(Timedelta::from_millis(-250))]
    #[case::the_longest(Timedelta::MAX)]
    fn a_span_crosses_both_ways(#[case] span: Timedelta) {
        let theirs = SignedDuration::from(span);
        assert_eq!(theirs.as_nanos(), i128::from(span.as_nanos()), "the same span");
        assert_eq!(Timedelta::try_from(theirs), Ok(span), "and back");
    }

    #[test]
    fn a_span_past_the_range_is_refused() {
        assert_eq!(Timedelta::try_from(SignedDuration::MAX), Err(OutOfRangeError), "forwards");
        assert_eq!(Timedelta::try_from(SignedDuration::MIN), Err(OutOfRangeError), "and backwards");
    }
}
