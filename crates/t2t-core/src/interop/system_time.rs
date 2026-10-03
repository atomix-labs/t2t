//! A [`Timestamp`] as std's `SystemTime`, and back, on 64-bit Linux and macOS.

use core::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::narrow;
use crate::{OutOfRangeError, Timestamp};

/// A `SystemTime` here counts 64-bit seconds, which reach past both ends of a [`Timestamp`].
impl From<Timestamp> for SystemTime {
    #[inline]
    fn from(instant: Timestamp) -> Self {
        let span = Duration::from_nanos(instant.0.unsigned_abs());
        let system_time =
            if instant.0 < 0 { UNIX_EPOCH.checked_sub(span) } else { UNIX_EPOCH.checked_add(span) };
        system_time.unwrap_or(UNIX_EPOCH)
    }
}

/// Refuses a `SystemTime` outside 1677-09-21 to 2262-04-11.
impl TryFrom<SystemTime> for Timestamp {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(instant: SystemTime) -> Result<Self, Self::Error> {
        let nanos = match instant.duration_since(UNIX_EPOCH) {
            Ok(span) => narrow(span.as_nanos()),
            Err(error) => narrow(0_i128.saturating_sub_unsigned(error.duration().as_nanos())),
        };
        nanos.map(Self)
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rstest::rstest;

    use crate::{OutOfRangeError, Timestamp};

    #[rstest]
    #[case::the_epoch(0)]
    #[case::a_nanosecond_after(1)]
    #[case::a_nanosecond_before(-1)]
    #[case::the_earliest(i64::MIN)]
    #[case::the_latest(i64::MAX)]
    fn an_instant_crosses_both_ways(#[case] nanos: i64) {
        let instant = Timestamp::from_nanos(nanos);
        assert_eq!(Timestamp::try_from(SystemTime::from(instant)), Ok(instant), "and back");
    }

    #[test]
    fn a_system_time_outside_the_range_is_refused() {
        let span = Duration::from_secs(1 << 40);
        let far_future = UNIX_EPOCH.checked_add(span).expect("a moment 35,000 years on");
        let distant_past = UNIX_EPOCH.checked_sub(span).expect("a moment 35,000 years back");
        assert_eq!(Timestamp::try_from(far_future), Err(OutOfRangeError), "after 2262");
        assert_eq!(Timestamp::try_from(distant_past), Err(OutOfRangeError), "before 1677");
    }
}
