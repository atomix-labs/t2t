//! A [`Timestamp`] written as RFC 3339 in UTC, and read back; a [`TaiTimestamp`] written the same
//! way, in its own zone.

use core::fmt;
use core::str::FromStr;

use crate::text::Text;
use crate::{ParseTimestampError, TaiTimestamp, Timestamp, UtcDateTime};

/// The digits of a fraction of a second.
const FRACTION_DIGITS: usize = 9;

/// The longest spelling, a [`TaiTimestamp`]'s with every fraction digit.
const LONGEST: usize = "2026-09-16T07:46:12.123456789 TAI".len();

/// What each digit of a fraction is worth in nanoseconds, the first to the ninth.
const FRACTION_SCALES: [u32; FRACTION_DIGITS] =
    [100_000_000, 10_000_000, 1_000_000, 100_000, 10_000, 1_000, 100, 10, 1];

/// The two ASCII digits of each number below 100.
const DIGIT_PAIRS: [[u8; 2]; 100] = {
    let mut pairs = [[0; 2]; 100];
    let mut number = 0;
    while number < 100 {
        #[expect(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::indexing_slicing,
            reason = "each digit is below 10, and `number` below 100; `TryFrom` and `get` are not const"
        )]
        {
            pairs[number] = [b'0' + (number / 10) as u8, b'0' + (number % 10) as u8];
            number += 1;
        }
    }
    pairs
};

/// RFC 3339 in UTC, with `Z`: `2026-09-16T07:45:35.123456789Z`.
///
/// The precision chooses the fraction's digits, nine by default and at most nine: `{:.3}` writes
/// milliseconds and `{:.0}` none.
impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_date_time(formatter, self.0, "Z")
    }
}

/// As [`Display`](fmt::Display) writes it.
impl fmt::Debug for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

/// RFC 3339's date and time, in the zone `TAI`: `2026-09-16T07:46:12.123456789 TAI`.
///
/// The precision chooses the fraction's digits, as for a [`Timestamp`].
impl fmt::Display for TaiTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_date_time(formatter, self.0, " TAI")
    }
}

/// As [`Display`](fmt::Display) writes it.
impl fmt::Debug for TaiTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

/// Writes `nanos` since 1970-01-01 as a date and time, the fraction to the formatter's precision,
/// then `zone`.
fn write_date_time(formatter: &mut fmt::Formatter<'_>, nanos: i64, zone: &str) -> fmt::Result {
    let digits = formatter.precision().unwrap_or(FRACTION_DIGITS).min(FRACTION_DIGITS);
    let date_time = Timestamp(nanos).to_utc();
    let mut text = Text::<LONGEST>::new();
    text.push(&date_time_bytes(date_time))?;
    if digits > 0 {
        let fraction = fraction_bytes(date_time.nanosecond);
        text.push(fraction.get(..=digits).ok_or(fmt::Error)?)?;
    }
    text.push(zone.as_bytes())?;
    fmt::Display::fmt(&text, formatter)
}

/// `date_time` to the second, `2026-09-16T07:45:35`; its year is a [`Timestamp`]'s, four digits.
fn date_time_bytes(date_time: UtcDateTime) -> [u8; 19] {
    let UtcDateTime { year, month, day, hour, minute, second, .. } = date_time;
    let year = year.cast_unsigned();
    with_pairs(
        *b"0000-00-00T00:00:00",
        [
            (0, year / 100),
            (2, year),
            (5, month.into()),
            (8, day.into()),
            (11, hour.into()),
            (14, minute.into()),
            (17, second.into()),
        ],
    )
}

/// A point, then the nine digits of `nanosecond`, below a billion: `.123456789`.
fn fraction_bytes(nanosecond: u32) -> [u8; 10] {
    let mut bytes = with_pairs(
        [0; 10],
        [
            (0, nanosecond / 100_000_000),
            (2, nanosecond / 1_000_000),
            (4, nanosecond / 10_000),
            (6, nanosecond / 100),
            (8, nanosecond),
        ],
    );
    // The first pair's tens digit, always zero, gives way to the point.
    bytes[0] = b'.';
    bytes
}

/// `template` with the last two digits of each `(offset, number)` written at its offset.
fn with_pairs<const LENGTH: usize, const COUNT: usize>(
    mut template: [u8; LENGTH], numbers: [(usize, u32); COUNT],
) -> [u8; LENGTH] {
    for (offset, number) in numbers {
        let pair = usize::try_from(number % 100).ok().and_then(|index| DIGIT_PAIRS.get(index));
        let slot = template.get_mut(offset..).and_then(<[u8]>::first_chunk_mut);
        if let (Some(slot), Some(pair)) = (slot, pair) {
            *slot = *pair;
        }
    }
    template
}

/// Reads what [`Display`](fmt::Display) writes, `T` and `Z` in either case, and a fraction of one
/// to nine digits or none.
///
/// It refuses an offset, a 60th second, a date that does not exist, and an instant outside
/// 1677-09-21 to 2262-04-11.
impl FromStr for Timestamp {
    type Err = ParseTimestampError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        parse(text).and_then(UtcDateTime::to_timestamp).ok_or(ParseTimestampError)
    }
}

/// Reads the whole of `text` as a date and time in UTC, without checking it is a real one.
#[expect(clippy::arithmetic_side_effects, reason = "four digits fit `i32`")]
fn parse(text: &str) -> Option<UtcDateTime> {
    let (date_time, rest) = text.as_bytes().split_first_chunk()?;
    let [
        year_thousands,
        year_hundreds,
        year_tens,
        year_ones,
        b'-',
        month_tens,
        month_ones,
        b'-',
        day_tens,
        day_ones,
        b'T' | b't',
        hour_tens,
        hour_ones,
        b':',
        minute_tens,
        minute_ones,
        b':',
        second_tens,
        second_ones,
    ] = *date_time
    else {
        return None;
    };
    let (nanosecond, zone) = fraction(rest)?;
    let [b'Z' | b'z'] = zone else {
        return None;
    };
    let century = two_digits(year_thousands, year_hundreds)?;
    let year_of_century = two_digits(year_tens, year_ones)?;
    Some(UtcDateTime {
        year: i32::from(century) * 100 + i32::from(year_of_century),
        month: two_digits(month_tens, month_ones)?,
        day: two_digits(day_tens, day_ones)?,
        hour: two_digits(hour_tens, hour_ones)?,
        minute: two_digits(minute_tens, minute_ones)?,
        second: two_digits(second_tens, second_ones)?,
        nanosecond,
    })
}

/// Two ASCII digits as a number, or `None` where either is not a digit.
#[expect(clippy::arithmetic_side_effects, reason = "two digits fit `u8`")]
fn two_digits(tens: u8, ones: u8) -> Option<u8> {
    let (tens, ones) = (tens.wrapping_sub(b'0'), ones.wrapping_sub(b'0'));
    (tens < 10 && ones < 10).then(|| tens * 10 + ones)
}

/// The nanoseconds of the fraction `text` opens with, `.` and one to nine digits, and the rest.
///
/// A `text` that opens with no point has no fraction: zero, and the whole `text`.
#[expect(clippy::arithmetic_side_effects, reason = "nine digits, each below its scale, fit `u32`")]
fn fraction(text: &[u8]) -> Option<(u32, &[u8])> {
    let Some(digits) = text.strip_prefix(b".") else {
        return Some((0, text));
    };
    let mut nanos = 0;
    let mut rest = digits;
    for scale in FRACTION_SCALES {
        let Some((byte, tail)) = rest.split_first() else {
            break;
        };
        let digit = byte.wrapping_sub(b'0');
        if digit >= 10 {
            break;
        }
        nanos += u32::from(digit) * scale;
        rest = tail;
    }
    let is_empty = rest.len() == digits.len();
    let has_tenth_digit = rest.first().is_some_and(u8::is_ascii_digit);
    (!is_empty && !has_tenth_digit).then_some((nanos, rest))
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;

    use proptest::prelude::{any, prop_assert_eq, proptest};
    use rstest::rstest;

    use crate::{ParseTimestampError, TaiTimestamp, Timestamp};

    #[test]
    fn an_instant_is_written_with_the_precision_choosing_its_fraction() {
        let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
        assert_eq!(instant.to_string(), "2026-09-16T07:45:35.123456789Z");
        assert_eq!(format!("{instant:.3}"), "2026-09-16T07:45:35.123Z");
        assert_eq!(format!("{instant:.0}"), "2026-09-16T07:45:35Z");
        assert_eq!(format!("{instant:.12}"), instant.to_string(), "nine digits at most");
        assert_eq!(format!("[{instant:>24.0}]"), "[    2026-09-16T07:45:35Z]");
    }

    #[test]
    fn a_tai_instant_is_written_in_its_own_zone() {
        let instant = TaiTimestamp::from_nanos(1_789_544_772_123_456_789);
        assert_eq!(instant.to_string(), "2026-09-16T07:46:12.123456789 TAI");
        assert_eq!(format!("{instant:.0}"), "2026-09-16T07:46:12 TAI");
    }

    #[test]
    fn a_short_fraction_reads_as_its_leading_digits_and_either_case_is_read() {
        assert_eq!("1970-01-01T00:00:00.5Z".parse(), Ok(Timestamp::from_millis(500)));
        assert_eq!("1970-01-01t00:00:00z".parse(), Ok(Timestamp::UNIX_EPOCH));
    }

    #[rstest]
    #[case::the_earliest(Timestamp::MIN)]
    #[case::the_latest(Timestamp::MAX)]
    fn the_range_ends_read_back(#[case] instant: Timestamp) {
        assert_eq!(instant.to_string().parse(), Ok(instant));
    }

    #[rstest]
    #[case::empty("")]
    #[case::a_date_alone("2026-09-16")]
    #[case::no_zone("2026-09-16T07:45:35")]
    #[case::an_offset("2026-09-16T07:45:35+01:00")]
    #[case::trailing_text("2026-09-16T07:45:35Z ")]
    #[case::february_30th("2026-02-30T07:45:35Z")]
    #[case::the_24th_hour("2026-09-16T24:45:35Z")]
    #[case::a_leap_second("2026-09-16T07:45:60Z")]
    #[case::a_letter_for_a_digit("2026-09-1xT07:45:35Z")]
    #[case::a_point_without_digits("2026-09-16T07:45:35.Z")]
    #[case::ten_fraction_digits("2026-09-16T07:45:35.1234567890Z")]
    #[case::after_the_range("2263-01-01T00:00:00Z")]
    #[case::before_the_range("1677-01-01T00:00:00Z")]
    fn a_malformed_instant_is_refused(#[case] text: &str) {
        assert_eq!(text.parse::<Timestamp>(), Err(ParseTimestampError));
    }

    proptest! {
        #[test]
        fn every_instant_reads_back_from_its_spelling(nanos in any::<i64>()) {
            let instant = Timestamp::from_nanos(nanos);
            prop_assert_eq!(instant.to_string().parse(), Ok(instant));
        }

        #[test]
        fn whatever_any_text_reads_as_reads_back_the_same(text in "[0-9T:Zz.-]{0,32}") {
            if let Ok(instant) = text.parse::<Timestamp>() {
                prop_assert_eq!(instant.to_string().parse(), Ok(instant));
            }
        }
    }
}
