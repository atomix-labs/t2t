//! Why a string is not a time, and why a time does not fit another type.

use thiserror::Error;

/// Why a string is not a [`Timedelta`](crate::Timedelta).
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[error(
    "parse timedelta error: expected `0`, or counts with units from coarsest to finest, as `1m30s`"
)]
pub struct ParseTimedeltaError;

/// Why a string is not a [`Timestamp`](crate::Timestamp).
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[error(
    "parse timestamp error: expected RFC 3339 in UTC from 1677 to 2262, as `2026-09-16T07:45:35Z`"
)]
pub struct ParseTimestampError;

/// Why a time does not fit the type it was converted to: a negative span for a
/// [`Duration`](core::time::Duration), or a value past what a count of nanoseconds holds.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[error("out of range error: the time is outside what the target type holds")]
pub struct OutOfRangeError;

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use crate::{OutOfRangeError, ParseTimedeltaError, ParseTimestampError};

    #[test]
    fn each_error_says_what_was_expected() {
        assert_eq!(
            ParseTimedeltaError.to_string(),
            "parse timedelta error: expected `0`, or counts with units from coarsest to finest, as `1m30s`"
        );
        assert_eq!(
            ParseTimestampError.to_string(),
            "parse timestamp error: expected RFC 3339 in UTC from 1677 to 2262, as `2026-09-16T07:45:35Z`"
        );
        assert_eq!(
            OutOfRangeError.to_string(),
            "out of range error: the time is outside what the target type holds"
        );
    }
}
