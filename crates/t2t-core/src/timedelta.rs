//! A signed span of nanoseconds, and its spelling.

use core::fmt;
use core::str::FromStr;
use core::time::Duration;

use crate::text::Text;
use crate::{OutOfRangeError, ParseTimedeltaError};

/// Nanoseconds in a microsecond.
pub(crate) const NANOS_PER_MICRO: i64 = 1_000;
/// Nanoseconds in a millisecond.
pub(crate) const NANOS_PER_MILLI: i64 = 1_000_000;
/// Nanoseconds in a second.
pub(crate) const NANOS_PER_SEC: i64 = 1_000_000_000;
/// Nanoseconds in a minute.
pub(crate) const NANOS_PER_MINUTE: i64 = 60 * NANOS_PER_SEC;
/// Nanoseconds in an hour.
pub(crate) const NANOS_PER_HOUR: i64 = 60 * NANOS_PER_MINUTE;
/// Nanoseconds in a day of 24 hours.
pub(crate) const NANOS_PER_DAY: i64 = 24 * NANOS_PER_HOUR;

/// Each unit of the spelling, coarsest first: its suffix, and its nanoseconds.
const UNITS: [(&str, u64); 7] = [
    ("d", NANOS_PER_DAY.unsigned_abs()),
    ("h", NANOS_PER_HOUR.unsigned_abs()),
    ("m", NANOS_PER_MINUTE.unsigned_abs()),
    ("s", NANOS_PER_SEC.unsigned_abs()),
    ("ms", NANOS_PER_MILLI.unsigned_abs()),
    ("us", NANOS_PER_MICRO.unsigned_abs()),
    ("ns", 1),
];

/// The longest spelling, [`Timedelta::MIN`]'s.
const LONGEST: usize = "-106751d23h47m16s854ms775us808ns".len();

/// A signed span of nanoseconds: how far apart two [`Timestamp`](crate::Timestamp)s or two
/// [`Uptime`](crate::Uptime)s are.
///
/// Its operators saturate at [`MIN`](Self::MIN) and [`MAX`](Self::MAX), about 292 years either
/// way, and each has a `checked_*` twin. It is written coarsest unit first, the way a config spells
/// it, and parses back from the same spelling.
///
/// # Examples
/// ```
/// use t2t_core::Timedelta;
///
/// let budget = Timedelta::from_millis(250);
/// assert_eq!(budget.as_micros(), 250_000, "the same span, counted finer");
/// assert_eq!(budget.to_string(), "250ms", "written in its coarsest unit");
/// assert_eq!("1m0s250ms".parse(), Ok(Timedelta::MINUTE + budget), "and read back");
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct Timedelta(pub(crate) i64);

const _: () = assert!(size_of::<Timedelta>() == size_of::<i64>(), "an `i64`, and nothing else");

impl Timedelta {
    /// One nanosecond, the finest span the count resolves.
    pub const NANOSECOND: Self = Self(1);
    /// One microsecond.
    pub const MICROSECOND: Self = Self(NANOS_PER_MICRO);
    /// One millisecond.
    pub const MILLISECOND: Self = Self(NANOS_PER_MILLI);
    /// One second.
    pub const SECOND: Self = Self(NANOS_PER_SEC);
    /// One minute.
    pub const MINUTE: Self = Self(NANOS_PER_MINUTE);
    /// One hour.
    pub const HOUR: Self = Self(NANOS_PER_HOUR);
    /// One day of 24 hours; no calendar is consulted.
    pub const DAY: Self = Self(NANOS_PER_DAY);

    /// A span of `nanos` nanoseconds.
    #[inline]
    #[must_use]
    pub const fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    /// A span of `micros` microseconds, saturating.
    #[inline]
    #[must_use]
    pub const fn from_micros(micros: i64) -> Self {
        Self(micros.saturating_mul(NANOS_PER_MICRO))
    }

    /// A span of `millis` milliseconds, saturating.
    #[inline]
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis.saturating_mul(NANOS_PER_MILLI))
    }

    /// A span of `secs` seconds, saturating.
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs.saturating_mul(NANOS_PER_SEC))
    }

    /// A span of `mins` minutes, saturating.
    #[inline]
    #[must_use]
    pub const fn from_mins(mins: i64) -> Self {
        Self(mins.saturating_mul(NANOS_PER_MINUTE))
    }

    /// A span of `hours` hours, saturating.
    #[inline]
    #[must_use]
    pub const fn from_hours(hours: i64) -> Self {
        Self(hours.saturating_mul(NANOS_PER_HOUR))
    }

    /// A span of `days` days of 24 hours, saturating.
    #[inline]
    #[must_use]
    pub const fn from_days(days: i64) -> Self {
        Self(days.saturating_mul(NANOS_PER_DAY))
    }

    /// The span in nanoseconds.
    #[inline]
    #[must_use]
    pub const fn as_nanos(self) -> i64 {
        self.0
    }

    /// The span in whole microseconds, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_micros(self) -> i64 {
        self.0 / NANOS_PER_MICRO
    }

    /// The span in whole milliseconds, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0 / NANOS_PER_MILLI
    }

    /// The span in whole seconds, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> i64 {
        self.0 / NANOS_PER_SEC
    }

    /// The span in whole minutes, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_mins(self) -> i64 {
        self.0 / NANOS_PER_MINUTE
    }

    /// The span in whole hours, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_hours(self) -> i64 {
        self.0 / NANOS_PER_HOUR
    }

    /// The span in whole days of 24 hours, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_days(self) -> i64 {
        self.0 / NANOS_PER_DAY
    }

    /// The nanoseconds past the whole seconds, signed as the span is.
    #[inline]
    #[must_use]
    pub const fn subsec_nanos(self) -> i32 {
        subsecond_part(self.0, 1)
    }

    /// The whole microseconds past the whole seconds, signed as the span is.
    #[inline]
    #[must_use]
    pub const fn subsec_micros(self) -> i32 {
        subsecond_part(self.0, NANOS_PER_MICRO)
    }

    /// The whole milliseconds past the whole seconds, signed as the span is.
    #[inline]
    #[must_use]
    pub const fn subsec_millis(self) -> i32 {
        subsecond_part(self.0, NANOS_PER_MILLI)
    }
}

/// The part of `nanos` past its whole seconds, in whole units of `unit` nanoseconds.
#[expect(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    reason = "every unit is a nonzero constant, and a part of a second fits `i32`; `TryFrom` is not const"
)]
const fn subsecond_part(nanos: i64, unit: i64) -> i32 {
    (nanos % NANOS_PER_SEC / unit) as i32
}

/// Coarsest unit first, once each, `0s` for zero: `1d2h`, `1m0s250ms`, `-3us`.
impl fmt::Display for Timedelta {
    #[expect(clippy::arithmetic_side_effects, reason = "every unit is a nonzero constant")]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = Text::<LONGEST>::new();
        if self.0 == 0 {
            text.push(b"0s")?;
        } else if self.0 < 0 {
            text.push(b"-")?;
        }
        // Unsigned, since `MIN` has no positive twin.
        let mut remainder = self.0.unsigned_abs();
        let coarsest = UNITS.iter().position(|&(_, unit)| remainder >= unit).unwrap_or(0);
        let mut digits = itoa::Buffer::new();
        for &(suffix, unit) in UNITS.iter().skip(coarsest) {
            if remainder == 0 {
                break;
            }
            text.push(digits.format(remainder / unit).as_bytes())?;
            text.push(suffix.as_bytes())?;
            remainder %= unit;
        }
        fmt::Display::fmt(&text, formatter)
    }
}

/// As [`Display`](fmt::Display) writes it.
impl fmt::Debug for Timedelta {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

/// Reads what [`Display`](fmt::Display) writes, and the bare `0`: each unit at most once and
/// coarsest first, so one span has one reading.
impl FromStr for Timedelta {
    type Err = ParseTimedeltaError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        parse(text).ok_or(ParseTimedeltaError)
    }
}

/// Reads `text` as a span, or `None`.
fn parse(text: &str) -> Option<Timedelta> {
    if text == "0" {
        return Some(Timedelta::ZERO);
    }
    let (is_negative, mut rest) = text.strip_prefix('-').map_or((false, text), |rest| (true, rest));
    if rest.is_empty() {
        return None;
    }
    let mut magnitude: u64 = 0;
    let mut previous_unit = u64::MAX;
    while !rest.is_empty() {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        let (count, tail) = rest.split_at_checked(digits)?;
        let count: u64 = count.parse().ok()?;
        // Finest first, so `ms` is tried before `m` and `s`.
        let (unit, tail) = UNITS
            .iter()
            .rev()
            .find_map(|&(suffix, unit)| Some((unit, tail.strip_prefix(suffix)?)))?;
        if unit >= previous_unit {
            return None;
        }
        previous_unit = unit;
        magnitude = magnitude.checked_add(count.checked_mul(unit)?)?;
        rest = tail;
    }
    let nanos = if is_negative {
        0_i64.checked_sub_unsigned(magnitude)
    } else {
        i64::try_from(magnitude).ok()
    };
    nanos.map(Timedelta)
}

/// Refuses a negative span.
impl TryFrom<Timedelta> for Duration {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(span: Timedelta) -> Result<Self, Self::Error> {
        u64::try_from(span.0).ok().map(Self::from_nanos).ok_or(OutOfRangeError)
    }
}

/// Refuses a duration past [`Timedelta::MAX`].
impl TryFrom<Duration> for Timedelta {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        i64::try_from(duration.as_nanos()).ok().map(Self).ok_or(OutOfRangeError)
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;
    use core::time::Duration;

    use proptest::prelude::{any, prop_assert_eq, proptest};
    use rstest::rstest;

    use crate::{OutOfRangeError, ParseTimedeltaError, Timedelta};

    #[test]
    fn every_unit_scales_to_the_same_count() {
        assert_eq!(Timedelta::from_micros(1), Timedelta::MICROSECOND);
        assert_eq!(Timedelta::from_millis(1), Timedelta::MILLISECOND);
        assert_eq!(Timedelta::from_secs(1), Timedelta::SECOND);
        assert_eq!(Timedelta::from_mins(1), Timedelta::MINUTE);
        assert_eq!(Timedelta::from_hours(1), Timedelta::HOUR);
        assert_eq!(Timedelta::from_days(1), Timedelta::DAY);
        assert_eq!(Timedelta::from_days(2).as_hours(), 48);
        assert_eq!(Timedelta::from_secs(i64::MAX), Timedelta::MAX, "saturating");
    }

    #[test]
    fn accessors_truncate_toward_zero() {
        let span = Timedelta::from_nanos(-1_234_567_890);
        assert_eq!(span.as_secs(), -1);
        assert_eq!(span.as_millis(), -1_234);
        assert_eq!(span.subsec_nanos(), -234_567_890);
        assert_eq!(span.subsec_micros(), -234_567);
        assert_eq!(span.subsec_millis(), -234);
    }

    #[rstest]
    #[case::zero(0, "0s")]
    #[case::a_day(86_400_000_000_000, "1d")]
    #[case::a_day_and_an_hour(90_000_000_000_000, "1d1h")]
    #[case::a_gap_inside(60_123_000_000, "1m0s123ms")]
    #[case::every_finer_unit(67_123_456_789, "1m7s123ms456us789ns")]
    #[case::backwards(-3_000, "-3us")]
    #[case::the_longest_backwards(i64::MIN, "-106751d23h47m16s854ms775us808ns")]
    fn a_span_is_spelled_coarsest_unit_first(#[case] nanos: i64, #[case] text: &str) {
        let span = Timedelta::from_nanos(nanos);
        assert_eq!(span.to_string(), text, "written");
        assert_eq!(text.parse(), Ok(span), "and read back");
    }

    #[test]
    fn a_reading_may_skip_a_coarser_unit_or_be_a_bare_zero() {
        assert_eq!("0".parse(), Ok(Timedelta::ZERO));
        assert_eq!("48h".parse(), Ok(Timedelta::from_days(2)));
    }

    #[test]
    fn a_width_pads_the_spelling() {
        let span = Timedelta::from_millis(500);
        assert_eq!(format!("[{span:8}]"), "[500ms   ]");
        assert_eq!(format!("[{span:>8}]"), "[   500ms]");
        assert_eq!(format!("[{span:-^9}]"), "[--500ms--]");
    }

    #[rstest]
    #[case::empty("")]
    #[case::a_sign_alone("-")]
    #[case::no_unit("1")]
    #[case::no_count("s")]
    #[case::a_unit_twice("1m1m")]
    #[case::finest_first("1ns2s")]
    #[case::spaced("5 minutes")]
    #[case::an_unknown_unit("1msec")]
    #[case::past_the_range("9223372036854775808ns")]
    fn a_malformed_span_is_refused(#[case] text: &str) {
        assert_eq!(text.parse::<Timedelta>(), Err(ParseTimedeltaError));
    }

    #[test]
    fn a_duration_crosses_both_ways_and_a_negative_span_is_refused() {
        assert_eq!(Duration::try_from(Timedelta::from_millis(5)), Ok(Duration::from_millis(5)));
        assert_eq!(Duration::try_from(Timedelta::from_millis(-5)), Err(OutOfRangeError));
        assert_eq!(Timedelta::try_from(Duration::from_millis(5)), Ok(Timedelta::from_millis(5)));
        assert_eq!(Timedelta::try_from(Duration::MAX), Err(OutOfRangeError));
    }

    proptest! {
        #[test]
        fn every_span_reads_back_from_its_spelling(nanos in any::<i64>()) {
            let span = Timedelta::from_nanos(nanos);
            prop_assert_eq!(span.to_string().parse(), Ok(span));
        }

        #[test]
        fn whatever_any_text_reads_as_reads_back_the_same(text in ".{0,40}") {
            if let Ok(span) = text.parse::<Timedelta>() {
                prop_assert_eq!(span.to_string().parse(), Ok(span));
            }
        }
    }
}
