//! [`Timestamp`] and [`Timedelta`] to and from jiff's `Timestamp` and `SignedDuration`.

use jiff::SignedDuration;

use crate::{OutOfRangeError, Timedelta, Timestamp};

/// Every timestamp, since jiff's reach years -9999 to 9999.
impl From<Timestamp> for jiff::Timestamp {
    #[inline]
    fn from(instant: Timestamp) -> Self {
        // jiff refuses only an instant past its years, which no timestamp reaches.
        Self::from_nanosecond(i128::from(instant.0)).unwrap_or(Self::UNIX_EPOCH)
    }
}

/// Refuses an instant outside [`Timestamp::MIN`] to [`Timestamp::MAX`].
impl TryFrom<jiff::Timestamp> for Timestamp {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(instant: jiff::Timestamp) -> Result<Self, Self::Error> {
        i64::try_from(instant.as_nanosecond()).ok().map(Self).ok_or(OutOfRangeError)
    }
}

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
        i64::try_from(duration.as_nanos()).ok().map(Self).ok_or(OutOfRangeError)
    }
}

#[cfg(test)]
mod tests {
    use jiff::SignedDuration;
    use rstest::rstest;

    use crate::{OutOfRangeError, Timedelta, Timestamp};

    #[rstest]
    #[case::the_earliest(Timestamp::MIN)]
    #[case::before_the_epoch(Timestamp::from_nanos(-1))]
    #[case::a_capture(Timestamp::from_nanos(1_789_544_735_123_456_789))]
    #[case::the_latest(Timestamp::MAX)]
    fn an_instant_crosses_both_ways(#[case] instant: Timestamp) {
        let theirs = jiff::Timestamp::from(instant);
        assert_eq!(theirs.as_nanosecond(), i128::from(instant.as_nanos()), "the same instant");
        assert_eq!(Timestamp::try_from(theirs), Ok(instant), "and back");
    }

    #[test]
    fn an_instant_past_the_range_is_refused() {
        assert_eq!(Timestamp::try_from(jiff::Timestamp::MAX), Err(OutOfRangeError));
        assert_eq!(Timestamp::try_from(jiff::Timestamp::MIN), Err(OutOfRangeError));
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
        assert_eq!(Timedelta::try_from(SignedDuration::MAX), Err(OutOfRangeError));
        assert_eq!(Timedelta::try_from(SignedDuration::MIN), Err(OutOfRangeError));
    }
}
