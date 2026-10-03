//! A [`Timestamp`] written as RFC 3339 in UTC, and read back; a [`TaiTimestamp`] written and read
//! the same way, in its own zone.

use core::fmt::{self, Write as _};
use core::str::FromStr;

use arrayvec::ArrayString;
use itoa::Buffer;

use crate::spelling::pad;
use crate::{ParseTaiTimestampError, ParseTimestampError, TaiTimestamp, Timestamp, UtcDateTime};

/// The digits of a fraction of a second.
const FRACTION_DIGITS: usize = 9;

/// The longest spelling, a [`TaiTimestamp`]'s with every fraction digit.
const LONGEST: usize = "2026-09-16T07:46:12.123456789 TAI".len();

/// Nanoseconds in a second: a count of nanoseconds past a second, plus it, has ten digits, the
/// first a `1`.
const NANOS_PER_SECOND: u32 = 1_000_000_000;

/// What each digit of a fraction is worth in nanoseconds, the first to the ninth.
const FRACTION_SCALES: [u32; FRACTION_DIGITS] =
    [100_000_000, 10_000_000, 1_000_000, 100_000, 10_000, 1_000, 100, 10, 1];

/// Each number below 100 as its two ASCII digits: an instant is written four and a half times as
/// fast through it as through `write!`'s `{:02}`, 24 µs against 109 µs for 1,024 on a Graviton4,
/// measured under `benches/results/2026-10-03T08-54Z-635b659-calendar-and-digit-pairs`.
#[rustfmt::skip]
const DIGIT_PAIRS: [&str; 100] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09",
    "10", "11", "12", "13", "14", "15", "16", "17", "18", "19",
    "20", "21", "22", "23", "24", "25", "26", "27", "28", "29",
    "30", "31", "32", "33", "34", "35", "36", "37", "38", "39",
    "40", "41", "42", "43", "44", "45", "46", "47", "48", "49",
    "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
    "60", "61", "62", "63", "64", "65", "66", "67", "68", "69",
    "70", "71", "72", "73", "74", "75", "76", "77", "78", "79",
    "80", "81", "82", "83", "84", "85", "86", "87", "88", "89",
    "90", "91", "92", "93", "94", "95", "96", "97", "98", "99",
];

/// The UTC zone, as [`Display`](fmt::Display) writes it.
const UTC: &str = "Z";

/// The TAI zone, as [`Display`](fmt::Display) writes it.
const TAI: &str = " TAI";

/// RFC 3339 in UTC, with `Z`: `2026-09-16T07:45:35.123456789Z`.
///
/// The precision chooses the fraction's digits, nine by default and at most nine: `{:.3}` writes
/// milliseconds and `{:.0}` none. A width pads the spelling.
impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_date_time(formatter, *self, UTC)
    }
}

/// RFC 3339's date and time, in the zone `TAI`: `2026-09-16T07:46:12.123456789 TAI`.
///
/// The precision chooses the fraction's digits, and a width pads the spelling, as for a
/// [`Timestamp`].
impl fmt::Display for TaiTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_date_time(formatter, Timestamp(self.0), TAI)
    }
}

/// Writes the date and time `instant` names, the fraction to the formatter's precision, then
/// `zone`.
fn write_date_time(
    formatter: &mut fmt::Formatter<'_>, instant: Timestamp, zone: &str,
) -> fmt::Result {
    let digits = formatter.precision().unwrap_or(FRACTION_DIGITS).min(FRACTION_DIGITS);
    let UtcDateTime { year, month, day, hour, minute, second, nanosecond } = instant.to_utc();
    // A timestamp's year has four digits, from 1677 to 2262.
    let year = year.cast_unsigned();
    let mut text = ArrayString::<LONGEST>::new();
    for (separator, number) in [
        ("", year / 100),
        ("", year),
        ("-", month.into()),
        ("-", day.into()),
        ("T", hour.into()),
        (":", minute.into()),
        (":", second.into()),
    ] {
        text.write_str(separator)?;
        text.write_str(pair(number))?;
    }
    if digits > 0 {
        #[expect(clippy::arithmetic_side_effects, reason = "below two billion, which fits `u32`")]
        let shifted = NANOS_PER_SECOND + nanosecond;
        let mut buffer = Buffer::new();
        let fraction = buffer.format(shifted).get(1..=digits).ok_or(fmt::Error)?;
        text.write_str(".")?;
        text.write_str(fraction)?;
    }
    text.write_str(zone)?;
    pad(formatter, &text)
}

/// The last two digits of `number`.
#[inline]
#[expect(
    clippy::indexing_slicing,
    clippy::as_conversions,
    reason = "a remainder of 100, widened, indexes a table of 100; `From` takes no `u32`"
)]
const fn pair(number: u32) -> &'static str {
    DIGIT_PAIRS[(number % 100) as usize]
}

/// Reads what [`Display`](fmt::Display) writes, `T` and `Z` in either case, and a fraction of one
/// to nine digits or none.
///
/// It refuses an offset, a 60th second, a date that does not exist, and an instant outside
/// 1677-09-21 to 2262-04-11.
impl FromStr for Timestamp {
    type Err = ParseTimestampError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let instant = parse(text)
            .filter(|&(_, zone)| matches!(zone, [b'Z' | b'z']))
            .and_then(|(date_time, _)| date_time.to_timestamp());
        instant.ok_or(ParseTimestampError)
    }
}

/// Reads what [`Display`](fmt::Display) writes: RFC 3339's date and time as a [`Timestamp`]
/// reads them, then the zone ` TAI`.
impl FromStr for TaiTimestamp {
    type Err = ParseTaiTimestampError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let instant = parse(text)
            .filter(|&(_, zone)| zone == TAI.as_bytes())
            .and_then(|(date_time, _)| date_time.to_timestamp());
        instant.map(|instant| Self(instant.0)).ok_or(ParseTaiTimestampError)
    }
}

/// Reads the date and time `text` opens with, without checking it is a real one, and the zone
/// after it.
#[expect(clippy::arithmetic_side_effects, reason = "four digits fit `i32`")]
fn parse(text: &str) -> Option<(UtcDateTime, &[u8])> {
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
    let century = two_digits(year_thousands, year_hundreds)?;
    let year_of_century = two_digits(year_tens, year_ones)?;
    let date_time = UtcDateTime {
        year: i32::from(century) * 100 + i32::from(year_of_century),
        month: two_digits(month_tens, month_ones)?,
        day: two_digits(day_tens, day_ones)?,
        hour: two_digits(hour_tens, hour_ones)?,
        minute: two_digits(minute_tens, minute_ones)?,
        second: two_digits(second_tens, second_ones)?,
        nanosecond,
    };
    Some((date_time, zone))
}

/// Two ASCII digits as a number, or `None` where either is not a digit.
#[inline]
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
    let has_digits = rest.len() < digits.len();
    let has_tenth_digit = rest.first().is_some_and(u8::is_ascii_digit);
    (has_digits && !has_tenth_digit).then_some((nanos, rest))
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;

    use proptest::prelude::{any, prop_assert_eq, proptest};
    use rstest::rstest;

    use crate::{ParseTaiTimestampError, ParseTimestampError, TaiTimestamp, Timestamp};

    #[test]
    fn an_instant_is_written_with_the_precision_choosing_its_fraction() {
        let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
        assert_eq!(instant.to_string(), "2026-09-16T07:45:35.123456789Z", "nine digits");
        assert_eq!(format!("{instant:.3}"), "2026-09-16T07:45:35.123Z", "three");
        assert_eq!(format!("{instant:.0}"), "2026-09-16T07:45:35Z", "none");
        assert_eq!(format!("{instant:.12}"), instant.to_string(), "and nine at most");
        let padded = format!("[{instant:>24.0}]");
        assert_eq!(padded, "[    2026-09-16T07:45:35Z]", "a width pads, and never cuts");
    }

    #[test]
    fn a_tai_instant_is_written_and_read_in_its_own_zone() {
        let instant = TaiTimestamp::from_nanos(1_789_544_772_123_456_789);
        assert_eq!(instant.to_string(), "2026-09-16T07:46:12.123456789 TAI", "written");
        assert_eq!(format!("{instant:.0}"), "2026-09-16T07:46:12 TAI", "to the precision");
        assert_eq!("2026-09-16T07:46:12.123456789 TAI".parse(), Ok(instant), "and read back");
        let utc = "2026-09-16T07:46:12Z".parse::<TaiTimestamp>();
        assert_eq!(utc, Err(ParseTaiTimestampError), "UTC is not its zone");
    }

    #[test]
    fn a_short_fraction_reads_as_its_leading_digits_and_either_case_is_read() {
        let half = "1970-01-01T00:00:00.5Z".parse();
        assert_eq!(half, Ok(Timestamp::from_millis(500)), "a digit worth tenths");
        assert_eq!("1970-01-01t00:00:00z".parse(), Ok(Timestamp::UNIX_EPOCH), "a lower case");
    }

    #[rstest]
    #[case::the_earliest(Timestamp::MIN)]
    #[case::the_latest(Timestamp::MAX)]
    fn the_range_ends_read_back(#[case] instant: Timestamp) {
        assert_eq!(instant.to_string().parse(), Ok(instant), "{instant} reads back");
    }

    #[rstest]
    #[case::empty("")]
    #[case::a_date_alone("2026-09-16")]
    #[case::no_zone("2026-09-16T07:45:35")]
    #[case::an_offset("2026-09-16T07:45:35+01:00")]
    #[case::the_tai_zone("2026-09-16T07:45:35 TAI")]
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
        assert_eq!(text.parse::<Timestamp>(), Err(ParseTimestampError), "{text:?} is refused");
    }

    proptest! {
        #[test]
        fn every_instant_reads_back_from_its_spelling(nanos in any::<i64>()) {
            let instant = Timestamp::from_nanos(nanos);
            prop_assert_eq!(instant.to_string().parse(), Ok(instant), "in UTC");
            let tai = TaiTimestamp::from_nanos(nanos);
            prop_assert_eq!(tai.to_string().parse(), Ok(tai), "and in TAI");
        }

        #[test]
        fn whatever_any_text_reads_as_reads_back_the_same(text in "[0-9T:Zz.-]{0,32}") {
            if let Ok(instant) = text.parse::<Timestamp>() {
                prop_assert_eq!(instant.to_string().parse(), Ok(instant), "what was read writes back");
            }
        }
    }
}
