//! A signed span of nanoseconds, and its spelling.

use core::fmt::{self, Write as _};
use core::str::FromStr;
use core::time::Duration;

use arrayvec::ArrayString;
use derive_more::Debug;
use itoa::Buffer;
#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::consts::{
    NANOS_PER_DAY, NANOS_PER_HOUR, NANOS_PER_MICROSECOND, NANOS_PER_MILLISECOND, NANOS_PER_MINUTE,
    NANOS_PER_SECOND,
};
use crate::spelling::pad;
use crate::{OutOfRangeError, ParseTimedeltaError, TickRate, Ticks};

/// The longest spelling, [`Timedelta::MIN`]'s.
const LONGEST: usize = "-106751d23h47m16s854ms775us808ns".len();

/// A signed span of nanoseconds: how far apart two points on one timeline are.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct Timedelta(pub(crate) i64);

impl Timedelta {
    /// One nanosecond, the finest span the count resolves.
    pub const NANOSECOND: Self = Self(1);
    /// One microsecond.
    pub const MICROSECOND: Self = Self(NANOS_PER_MICROSECOND);
    /// One millisecond.
    pub const MILLISECOND: Self = Self(NANOS_PER_MILLISECOND);
    /// One second.
    pub const SECOND: Self = Self(NANOS_PER_SECOND);
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
        Self(micros.saturating_mul(NANOS_PER_MICROSECOND))
    }

    /// A span of `millis` milliseconds, saturating.
    #[inline]
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis.saturating_mul(NANOS_PER_MILLISECOND))
    }

    /// A span of `secs` seconds, saturating.
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs.saturating_mul(NANOS_PER_SECOND))
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
        self.0 / NANOS_PER_MICROSECOND
    }

    /// The span in whole milliseconds, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0 / NANOS_PER_MILLISECOND
    }

    /// The span in whole seconds, truncated toward zero.
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> i64 {
        self.0 / NANOS_PER_SECOND
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
        subsecond_part::<1>(self.0)
    }

    /// The whole microseconds past the whole seconds, signed as the span is.
    #[inline]
    #[must_use]
    pub const fn subsec_micros(self) -> i32 {
        subsecond_part::<NANOS_PER_MICROSECOND>(self.0)
    }

    /// The whole milliseconds past the whole seconds, signed as the span is.
    #[inline]
    #[must_use]
    pub const fn subsec_millis(self) -> i32 {
        subsecond_part::<NANOS_PER_MILLISECOND>(self.0)
    }

    /// How many ticks the span lasts at `rate`: a multiply and a shift, to within a tick.
    #[inline]
    #[must_use]
    pub const fn to_ticks(self, rate: TickRate) -> Ticks {
        Ticks(rate.nanos_to_ticks(self.0))
    }
}

/// The part of `nanos` past its whole seconds, in whole units of `UNIT` nanoseconds.
#[inline]
#[expect(
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    reason = "every unit is a nonzero constant, and a part of a second fits `i32`; `TryFrom` is not const"
)]
const fn subsecond_part<const UNIT: i64>(nanos: i64) -> i32 {
    (nanos % NANOS_PER_SECOND / UNIT) as i32
}

/// Coarsest unit first, once each, `0s` for zero: `1d2h`, `1m0s250ms`, `-3us`.
impl fmt::Display for Timedelta {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut spelling =
            Spelling { text: ArrayString::new(), remainder: self.0.unsigned_abs(), started: false };
        if self.0 == 0 {
            spelling.text.write_str("0s")?;
        } else if self.0 < 0 {
            spelling.text.write_str("-")?;
        }
        // A unit apiece, so each divides by a constant, which compiles to a multiply.
        spelling.write_unit::<{ NANOS_PER_DAY.unsigned_abs() }>("d")?;
        spelling.write_unit::<{ NANOS_PER_HOUR.unsigned_abs() }>("h")?;
        spelling.write_unit::<{ NANOS_PER_MINUTE.unsigned_abs() }>("m")?;
        spelling.write_unit::<{ NANOS_PER_SECOND.unsigned_abs() }>("s")?;
        spelling.write_unit::<{ NANOS_PER_MILLISECOND.unsigned_abs() }>("ms")?;
        spelling.write_unit::<{ NANOS_PER_MICROSECOND.unsigned_abs() }>("us")?;
        spelling.write_unit::<1>("ns")?;
        pad(formatter, &spelling.text)
    }
}

/// A span's spelling, as far as it is written.
struct Spelling {
    /// What is written so far.
    text: ArrayString<LONGEST>,
    /// The nanoseconds left to write: unsigned, since `MIN` has no positive twin.
    remainder: u64,
    /// Whether a unit is written, after which each finer one is, a zero among them.
    started: bool,
}

impl Spelling {
    /// Writes the whole `UNIT`s left, then `suffix`, once a coarser unit is written or one is
    /// whole, and keeps what is finer; nothing, once nothing is left.
    #[inline]
    #[expect(clippy::arithmetic_side_effects, reason = "`UNIT` is a nonzero constant")]
    fn write_unit<const UNIT: u64>(&mut self, suffix: &str) -> fmt::Result {
        if self.remainder == 0 {
            return Ok(());
        }
        let count = self.remainder / UNIT;
        if self.started || count > 0 {
            self.text.write_str(Buffer::new().format(count))?;
            self.text.write_str(suffix)?;
            self.started = true;
        }
        self.remainder %= UNIT;
        Ok(())
    }
}

/// Reads what [`Display`](fmt::Display) writes, and the bare `0`: each unit at most once and
/// coarsest first, with any count in each, so `48h` reads as the span `2d` spells.
impl FromStr for Timedelta {
    type Err = ParseTimedeltaError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        parse(text.as_bytes()).ok_or(ParseTimedeltaError)
    }
}

/// Reads `text` as a span in one pass, or `None`.
fn parse(text: &[u8]) -> Option<Timedelta> {
    if text == b"0" {
        return Some(Timedelta::ZERO);
    }
    let (is_negative, mut rest) = match text {
        [b'-', rest @ ..] => (true, rest),
        _ => (false, text),
    };
    if rest.is_empty() {
        return None;
    }
    let mut magnitude: u64 = 0;
    let mut previous_unit = u64::MAX;
    while !rest.is_empty() {
        let (count, tail) = read_digits(rest)?;
        // The units `Display` writes, `ms` tried before `m`.
        let (unit, tail) = match tail {
            [b'd', tail @ ..] => (NANOS_PER_DAY, tail),
            [b'h', tail @ ..] => (NANOS_PER_HOUR, tail),
            [b'm', b's', tail @ ..] => (NANOS_PER_MILLISECOND, tail),
            [b'm', tail @ ..] => (NANOS_PER_MINUTE, tail),
            [b's', tail @ ..] => (NANOS_PER_SECOND, tail),
            [b'u', b's', tail @ ..] => (NANOS_PER_MICROSECOND, tail),
            [b'n', b's', tail @ ..] => (1, tail),
            _ => return None,
        };
        let unit = unit.unsigned_abs();
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

/// The count `text` opens with, one ASCII digit at least, and the text after it.
fn read_digits(text: &[u8]) -> Option<(u64, &[u8])> {
    let mut count: u64 = 0;
    let mut rest = text;
    while let [digit @ b'0'..=b'9', tail @ ..] = rest {
        count = count.checked_mul(10)?.checked_add(u64::from(digit.wrapping_sub(b'0')))?;
        rest = tail;
    }
    (rest.len() < text.len()).then_some((count, rest))
}

/// Refuses a negative span.
impl TryFrom<Timedelta> for Duration {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(span: Timedelta) -> Result<Self, Self::Error> {
        u64::try_from(span.0).map(Self::from_nanos).map_err(|_past_the_range| OutOfRangeError)
    }
}

/// Refuses a duration past [`Timedelta::MAX`].
impl TryFrom<Duration> for Timedelta {
    type Error = OutOfRangeError;

    #[inline]
    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        i64::try_from(duration.as_nanos()).map(Self).map_err(|_past_the_range| OutOfRangeError)
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
        assert_eq!(Timedelta::from_micros(1), Timedelta::MICROSECOND, "a microsecond");
        assert_eq!(Timedelta::from_millis(1), Timedelta::MILLISECOND, "a millisecond");
        assert_eq!(Timedelta::from_secs(1), Timedelta::SECOND, "a second");
        assert_eq!(Timedelta::from_mins(1), Timedelta::MINUTE, "a minute");
        assert_eq!(Timedelta::from_hours(1), Timedelta::HOUR, "an hour");
        assert_eq!(Timedelta::from_days(1), Timedelta::DAY, "a day");
        assert_eq!(Timedelta::from_days(2).as_hours(), 48, "a coarser unit, counted finer");
        assert_eq!(
            Timedelta::from_secs(i64::MAX),
            Timedelta::MAX,
            "a count past the range saturates"
        );
    }

    #[test]
    fn a_backwards_span_counts_its_whole_units_toward_zero() {
        let span = Timedelta::from_nanos(-1_234_567_890);
        assert_eq!(span.as_secs(), -1, "whole seconds");
        assert_eq!(span.as_millis(), -1_234, "whole milliseconds");
        assert_eq!(span.subsec_nanos(), -234_567_890, "the nanoseconds past the seconds");
        assert_eq!(span.subsec_micros(), -234_567, "the microseconds past them");
        assert_eq!(span.subsec_millis(), -234, "and the milliseconds");
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
        assert_eq!("0".parse(), Ok(Timedelta::ZERO), "a bare zero");
        assert_eq!("48h".parse(), Ok(Timedelta::from_days(2)), "a count past the next unit up");
    }

    #[test]
    fn a_width_pads_the_spelling() {
        let span = Timedelta::from_millis(500);
        assert_eq!(format!("[{span:8}]"), "[500ms   ]", "to the left by default");
        assert_eq!(format!("[{span:>8}]"), "[   500ms]", "to the right");
        assert_eq!(format!("[{span:-^9}]"), "[--500ms--]", "and centred, with a fill");
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
    #[case::a_backwards_zero("-0")]
    #[case::a_sign_display_never_writes("+1s")]
    #[case::past_the_range("9223372036854775808ns")]
    #[case::a_count_past_any_unit("99999999999999999999999ns")]
    fn a_malformed_span_is_refused(#[case] text: &str) {
        assert_eq!(text.parse::<Timedelta>(), Err(ParseTimedeltaError), "{text:?} is refused");
    }

    #[test]
    fn a_duration_crosses_both_ways_and_a_negative_span_is_refused() {
        let (span, duration) = (Timedelta::from_millis(5), Duration::from_millis(5));
        assert_eq!(Duration::try_from(span), Ok(duration), "a span to a duration");
        assert_eq!(Timedelta::try_from(duration), Ok(span), "and back");
        assert_eq!(Duration::try_from(-span), Err(OutOfRangeError), "no duration runs backwards");
        assert_eq!(Timedelta::try_from(Duration::MAX), Err(OutOfRangeError), "nor past the range");
    }

    proptest! {
        #[test]
        fn every_span_reads_back_from_its_spelling(nanos in any::<i64>()) {
            let span = Timedelta::from_nanos(nanos);
            prop_assert_eq!(span.to_string().parse(), Ok(span), "the spelling reads back");
        }

        #[test]
        fn whatever_any_text_reads_as_reads_back_the_same(text in ".{0,40}") {
            if let Ok(span) = text.parse::<Timedelta>() {
                prop_assert_eq!(span.to_string().parse(), Ok(span), "what was read writes back");
            }
        }
    }
}
