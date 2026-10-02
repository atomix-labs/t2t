//! A point on the wall clock as a date and a time of day.

#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::Timestamp;
use crate::consts::{
    NANOS_PER_DAY, NANOS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE,
};

/// The days in 400 years of the Gregorian calendar, the cycle it repeats in.
const DAYS_PER_CYCLE: i64 = 146_097;
/// The days from 0000-03-01, where Neri and Schneider's calendar starts, to 1970-01-01.
const EPOCH_DAY: i64 = 719_468;
/// The first day a [`Timestamp`] reaches, 1677-09-21, counted from 1970-01-01.
const FIRST_DAY: i64 = Timestamp::MIN.0.div_euclid(NANOS_PER_DAY);
/// The 400-year cycles the calendar is shifted by, so every year an `i32` holds counts up from
/// zero, where Neri and Schneider's algorithm works.
const CYCLE_SHIFT: i64 = 5_368_710;
/// The years in [`CYCLE_SHIFT`].
const YEAR_SHIFT: i64 = 400 * CYCLE_SHIFT;
/// The days from the shifted calendar's origin, a March 1st, to 1970-01-01.
const DAY_SHIFT: i64 = EPOCH_DAY + DAYS_PER_CYCLE * CYCLE_SHIFT;

/// A point on the wall clock read as a date and a time of day: UTC, in the proleptic Gregorian
/// calendar.
///
/// A view of a point: arithmetic stays on the [`Timestamp`] it came from. Unix time has no leap
/// seconds, so `second` is never 60. The fields are public, so one may be built by hand;
/// [`is_valid`](Self::is_valid) checks one.
///
/// # Examples
/// ```
/// use t2t_core::{Timestamp, UtcDateTime};
///
/// let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
/// let date_time = instant.to_utc();
///
/// assert_eq!((date_time.year, date_time.month, date_time.day), (2026, 9, 16), "the date");
/// assert_eq!(
///     (date_time.hour, date_time.minute, date_time.second),
///     (7, 45, 35),
///     "the time of day"
/// );
/// assert_eq!(date_time.nanosecond, 123_456_789, "to the nanosecond");
/// assert_eq!(date_time.to_timestamp(), Some(instant), "and back to its instant");
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, Immutable, KnownLayout))]
pub struct UtcDateTime {
    /// The year.
    pub year: i32,
    /// The month of the year, from 1.
    pub month: u8,
    /// The day of the month, from 1.
    pub day: u8,
    /// The hour of the day, below 24.
    pub hour: u8,
    /// The minute of the hour, below 60.
    pub minute: u8,
    /// The second of the minute, below 60.
    pub second: u8,
    /// The nanoseconds past the second, below a billion.
    pub nanosecond: u32,
}

impl UtcDateTime {
    /// The point this names, or `None` for one that is not [valid](Self::is_valid) or is
    /// outside 1677-09-21 to 2262-04-11.
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "a date and time's nanoseconds fit `i128`, and the sum is checked to fit `i64`; `From` \
                  and `TryFrom` are not const"
    )]
    pub const fn to_timestamp(self) -> Option<Timestamp> {
        if !self.is_valid() {
            return None;
        }
        // In `i128`, since the second holding `Timestamp::MIN` starts below it.
        let nanos =
            seconds_from_civil(self) as i128 * NANOS_PER_SECOND as i128 + self.nanosecond as i128;
        if nanos < i64::MIN as i128 || nanos > i64::MAX as i128 {
            return None;
        }
        Some(Timestamp(nanos as i64))
    }

    /// Whether every field names a real date and time.
    #[must_use]
    #[expect(clippy::as_conversions, reason = "widening is lossless; `From` is not const")]
    pub const fn is_valid(self) -> bool {
        self.month >= 1
            && self.month <= 12
            && self.day >= 1
            && self.day <= days_in_month(self.year, self.month)
            && self.hour < 24
            && self.minute < 60
            && self.second < 60
            && (self.nanosecond as i64) < NANOS_PER_SECOND
    }
}

impl Timestamp {
    /// This point read as a date and a time of day in UTC.
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "counted from the first day a timestamp reaches, its seconds are nonnegative and \
                  inside `u64`, and its days inside `u32`; `TryFrom` is not const"
    )]
    pub const fn to_utc(self) -> UtcDateTime {
        // Counted from `FIRST_DAY`, every split is an unsigned division by a constant, and the
        // time of day's in 32 bits.
        let seconds_per_day = SECONDS_PER_DAY.unsigned_abs();
        let seconds_per_hour = SECONDS_PER_HOUR as u32;
        let seconds_per_minute = SECONDS_PER_MINUTE as u32;
        let secs = (self.0.div_euclid(NANOS_PER_SECOND) - FIRST_DAY * SECONDS_PER_DAY) as u64;
        let day = (secs / seconds_per_day) as u32 + (EPOCH_DAY + FIRST_DAY) as u32;
        let second_of_day = (secs % seconds_per_day) as u32;
        let (year, month, day) = civil_from_day(day);
        UtcDateTime {
            year,
            month,
            day,
            hour: (second_of_day / seconds_per_hour) as u8,
            minute: (second_of_day % seconds_per_hour / seconds_per_minute) as u8,
            second: (second_of_day % seconds_per_minute) as u8,
            nanosecond: self.0.rem_euclid(NANOS_PER_SECOND) as u32,
        }
    }
}

impl From<Timestamp> for UtcDateTime {
    #[inline]
    fn from(instant: Timestamp) -> Self {
        instant.to_utc()
    }
}

/// Whether `year` has a February 29th.
const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// The days in `month` of `year`.
const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The seconds from 1970-01-01T00:00:00 to `date_time`, by Neri and Schneider's algorithm.
#[expect(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    reason = "every term stays far inside `i64` for any field values; `From` is not const"
)]
const fn seconds_from_civil(date_time: UtcDateTime) -> i64 {
    let is_january_or_february = date_time.month <= 2;
    let year = date_time.year as i64 + YEAR_SHIFT - is_january_or_february as i64;
    let month = date_time.month as i64 + if is_january_or_february { 12 } else { 0 };
    let century = year / 100;
    let year_days = 1_461 * year / 4 - century + century / 4;
    let month_days = (979 * month - 2_919) / 32;
    let days = year_days + month_days + date_time.day as i64 - 1 - DAY_SHIFT;
    days * SECONDS_PER_DAY
        + date_time.hour as i64 * SECONDS_PER_HOUR
        + date_time.minute as i64 * SECONDS_PER_MINUTE
        + date_time.second as i64
}

/// The date `day` days after 0000-03-01, a day a [`Timestamp`] reaches, by Neri and Schneider.
#[expect(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "a timestamp's day keeps every term inside `u32`, or `u64` for the year's product, and \
              each part inside its type; `From` and `TryFrom` are not const"
)]
const fn civil_from_day(day: u32) -> (i32, u8, u8) {
    let century_part = 4 * day + 3;
    let days_per_cycle = DAYS_PER_CYCLE as u32;
    let century = century_part / days_per_cycle;
    let day_of_century = century_part % days_per_cycle / 4;
    let year_part = 2_939_745 * (4 * day_of_century + 3) as u64;
    let year_of_century = (year_part >> 32) as u32;
    let day_of_year = year_part as u32 / 2_939_745 / 4;
    let month_part = 2_141 * day_of_year + 197_913;
    let day = (month_part & 0xFFFF) / 2_141 + 1;
    let is_january_or_february = day_of_year >= 306;
    let year = 100 * century + year_of_century + is_january_or_february as u32;
    let month = (month_part >> 16) - if is_january_or_february { 12 } else { 0 };
    (year as i32, month as u8, day as u8)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, prop_assert_eq, prop_oneof, proptest};

    use super::{DAYS_PER_CYCLE, EPOCH_DAY, civil_from_day};
    use crate::{Timestamp, UtcDateTime};

    /// The date `days` after 1970-01-01, by Howard Hinnant's `civil_from_days`, as a reference.
    #[expect(clippy::arithmetic_side_effects, reason = "a timestamp's day keeps every term small")]
    fn reference(days: i64) -> (i32, u8, u8) {
        let day_count = days.checked_add(EPOCH_DAY).expect("a timestamp's day");
        let era = day_count.div_euclid(DAYS_PER_CYCLE);
        let day_of_era = day_count.rem_euclid(DAYS_PER_CYCLE);
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_part = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_part + 2) / 5 + 1;
        let month = if month_part < 10 { month_part + 3 } else { month_part - 9 };
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        (
            i32::try_from(year).expect("a timestamp's year"),
            u8::try_from(month).expect("a month"),
            u8::try_from(day).expect("a day"),
        )
    }

    #[test]
    fn the_epoch_is_the_first_instant_of_1970() {
        let date_time = Timestamp::UNIX_EPOCH.to_utc();
        assert_eq!(
            date_time,
            UtcDateTime {
                year: 1970,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
                nanosecond: 0
            },
            "midnight on the first of January 1970"
        );
        assert_eq!(date_time.to_timestamp(), Some(Timestamp::UNIX_EPOCH), "and back");
    }

    #[test]
    fn an_instant_before_the_epoch_counts_its_fraction_forwards() {
        let date_time = Timestamp::from_nanos(-1).to_utc();
        let date = (date_time.year, date_time.month, date_time.day);
        assert_eq!(date, (1969, 12, 31), "the last day of 1969");
        let time = (date_time.hour, date_time.minute, date_time.second);
        assert_eq!(time, (23, 59, 59), "its last second");
        assert_eq!(date_time.nanosecond, 999_999_999, "and its last nanosecond");
    }

    #[test]
    fn the_range_ends_read_as_their_dates() {
        assert_eq!(Timestamp::MIN.to_utc().year, 1677, "the earliest in 1677");
        assert_eq!(Timestamp::MAX.to_utc().year, 2262, "the latest in 2262");
        let (earliest, latest) = (Timestamp::MIN.to_utc(), Timestamp::MAX.to_utc());
        assert_eq!(earliest.to_timestamp(), Some(Timestamp::MIN), "each read back");
        assert_eq!(latest.to_timestamp(), Some(Timestamp::MAX), "both");
    }

    #[test]
    fn a_date_outside_the_range_has_no_instant() {
        let date_time = Timestamp::UNIX_EPOCH.to_utc();
        assert_eq!(UtcDateTime { year: 3000, ..date_time }.to_timestamp(), None, "after 2262");
        assert_eq!(UtcDateTime { year: 1000, ..date_time }.to_timestamp(), None, "before 1677");
    }

    #[test]
    fn a_leap_day_is_valid_and_a_february_30th_is_not() {
        let leap_day = UtcDateTime {
            year: 2024,
            month: 2,
            day: 29,
            hour: 12,
            minute: 0,
            second: 0,
            nanosecond: 0,
        };
        assert!(leap_day.is_valid(), "a leap day exists");
        let round_trip = leap_day.to_timestamp().map(Timestamp::to_utc);
        assert_eq!(round_trip, Some(leap_day), "and reads back");
        assert_eq!(UtcDateTime { day: 30, ..leap_day }.to_timestamp(), None, "no February 30th");
        assert!(!UtcDateTime { year: 2023, ..leap_day }.is_valid(), "2023 is no leap year");
        assert!(!UtcDateTime { year: 1900, ..leap_day }.is_valid(), "nor is a century");
        assert!(UtcDateTime { year: 2000, ..leap_day }.is_valid(), "but every 400th year is");
        assert!(!UtcDateTime { day: 30, ..leap_day }.is_valid(), "no day past the month's end");
        assert!(!UtcDateTime { month: 13, ..leap_day }.is_valid(), "no thirteenth month");
        assert!(!UtcDateTime { second: 60, ..leap_day }.is_valid(), "no leap second");
        let past_a_second = UtcDateTime { nanosecond: 1_000_000_000, ..leap_day };
        assert!(!past_a_second.is_valid(), "no fraction of a whole second");
    }

    proptest! {
        #[test]
        fn the_calendar_agrees_with_the_reference(days in -106_752_i64..=106_751) {
            let day = days.checked_add(EPOCH_DAY).and_then(|day| u32::try_from(day).ok());
            prop_assert_eq!(day.map(civil_from_day), Some(reference(days)), "the same date");
        }

        #[test]
        fn every_instant_reads_back_from_its_date(nanos in any::<i64>()) {
            let instant = Timestamp::from_nanos(nanos);
            prop_assert_eq!(instant.to_utc().to_timestamp(), Some(instant), "the same instant");
        }

        // Every field a little past its range, and any year, as bytes or a hand may build them.
        #[test]
        fn any_fields_read_as_the_instant_they_name_or_as_none(
            year in prop_oneof![any::<i32>(), 1_600_i32..=2_300],
            month in 0_u8..=13,
            day in 0_u8..=32,
            hour in 0_u8..=24,
            minute in 0_u8..=60,
            second in 0_u8..=60,
            nanosecond in 0_u32..=1_000_000_000,
        ) {
            let date_time = UtcDateTime { year, month, day, hour, minute, second, nanosecond };
            if let Some(instant) = date_time.to_timestamp() {
                prop_assert_eq!(instant.to_utc(), date_time, "the instant reads back as its fields");
            }
        }
    }
}
