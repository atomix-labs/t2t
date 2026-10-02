//! Why a string is not a time, and why a time does not fit another type.

use derive_more::{Display, Error};

/// Why a string is not a [`Timedelta`](crate::Timedelta), nor a point counted from boot.
///
/// # Examples
/// ```
/// use t2t_core::{ParseTimedeltaError, Timedelta};
///
/// assert_eq!("90 seconds".parse::<Timedelta>(), Err(ParseTimedeltaError), "no such unit");
/// ```
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[display(
    "parse timedelta error: expected `0`, or counts with units from coarsest to finest, as `1m30s`"
)]
pub struct ParseTimedeltaError;

/// Why a string is not a [`Timestamp`](crate::Timestamp).
///
/// # Examples
/// ```
/// use t2t_core::{ParseTimestampError, Timestamp};
///
/// let local = "2026-09-16T09:45:35+02:00".parse::<Timestamp>();
/// assert_eq!(local, Err(ParseTimestampError), "an instant is read in UTC alone");
/// ```
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[display(
    "parse timestamp error: expected RFC 3339 in UTC from 1677 to 2262, as `2026-09-16T07:45:35Z`"
)]
pub struct ParseTimestampError;

/// Why a string is not a [`TaiTimestamp`](crate::TaiTimestamp).
///
/// # Examples
/// ```
/// use t2t_core::{ParseTaiTimestampError, TaiTimestamp};
///
/// let utc = "2026-09-16T07:46:12Z".parse::<TaiTimestamp>();
/// assert_eq!(utc, Err(ParseTaiTimestampError), "an instant in UTC is no instant in TAI");
/// ```
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[display(
    "parse TAI timestamp error: expected an RFC 3339 date and time in TAI from 1677 to 2262, as \
     `2026-09-16T07:46:12 TAI`"
)]
pub struct ParseTaiTimestampError;

/// Why a string is not a [`Ticks`](crate::Ticks), nor a [`Tick`](crate::Tick).
///
/// # Examples
/// ```
/// use t2t_core::{ParseTicksError, Ticks};
///
/// assert_eq!("24".parse::<Ticks>(), Err(ParseTicksError), "a count names its unit");
/// ```
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[display("parse ticks error: expected a count of ticks, as `24 ticks`")]
pub struct ParseTicksError;

/// Why a string is not a [`TickRate`](crate::TickRate).
///
/// # Examples
/// ```
/// use t2t_core::{ParseTickRateError, TickRate};
///
/// assert_eq!("0 Hz".parse::<TickRate>(), Err(ParseTickRateError), "a counter that never ticks");
/// ```
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[display("parse tick rate error: expected a count of hertz above zero, as `24000000 Hz`")]
pub struct ParseTickRateError;

/// Why a time does not fit the type it was converted to: a negative span for a
/// [`Duration`](core::time::Duration), or a value past what a count of nanoseconds holds.
///
/// # Examples
/// ```
/// use core::time::Duration;
///
/// use t2t_core::{OutOfRangeError, Timedelta};
///
/// let backwards = Duration::try_from(-Timedelta::SECOND);
/// assert_eq!(backwards, Err(OutOfRangeError), "a duration never runs backwards");
/// ```
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[display("out of range error: the time is outside what the target type holds")]
pub struct OutOfRangeError;

#[cfg(test)]
mod tests {
    use alloc::string::{String, ToString as _};

    use rstest::rstest;

    use crate::{
        OutOfRangeError, ParseTaiTimestampError, ParseTickRateError, ParseTicksError,
        ParseTimedeltaError, ParseTimestampError,
    };

    #[rstest]
    #[case::a_span(
        ParseTimedeltaError.to_string(),
        "parse timedelta error: expected `0`, or counts with units from coarsest to finest, as `1m30s`"
    )]
    #[case::an_instant(
        ParseTimestampError.to_string(),
        "parse timestamp error: expected RFC 3339 in UTC from 1677 to 2262, as `2026-09-16T07:45:35Z`"
    )]
    #[case::a_tai_instant(
        ParseTaiTimestampError.to_string(),
        "parse TAI timestamp error: expected an RFC 3339 date and time in TAI from 1677 to 2262, \
         as `2026-09-16T07:46:12 TAI`"
    )]
    #[case::a_count_of_ticks(
        ParseTicksError.to_string(),
        "parse ticks error: expected a count of ticks, as `24 ticks`"
    )]
    #[case::a_rate(
        ParseTickRateError.to_string(),
        "parse tick rate error: expected a count of hertz above zero, as `24000000 Hz`"
    )]
    #[case::a_conversion(
        OutOfRangeError.to_string(),
        "out of range error: the time is outside what the target type holds"
    )]
    fn each_refusal_says_why(#[case] message: String, #[case] text: &str) {
        assert_eq!(message, text, "the message names what would have been read");
    }
}
