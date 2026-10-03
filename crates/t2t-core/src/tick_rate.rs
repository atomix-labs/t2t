//! A counter's rate, and the conversion between its ticks and nanoseconds.

use core::num::NonZeroU64;
use core::str::FromStr;

use derive_more::{Debug, Display};
#[cfg(feature = "zerocopy")]
use zerocopy::{Immutable, KnownLayout};

use crate::ParseTickRateError;
use crate::consts::NANOS_PER_SECOND;
use crate::spelling::{Count, read_count};

/// How many ticks a counter advances in a second: what turns [`Tickdelta`](crate::Tickdelta) into a
/// [`Timedelta`](crate::Timedelta) and back.
///
/// A conversion is a multiply and a shift, with factors [`from_hertz`](Self::from_hertz) works out
/// once: no division. An exact multiple converts exactly, any other count to within one unit of
/// its exact quotient, and the range's ends saturate. It is written as its hertz, `24000000 Hz`,
/// and parses back from it.
///
/// # Examples
/// ```
/// use t2t_core::{TickRate, Tickdelta, Timedelta};
///
/// let rate: TickRate = "24000000 Hz".parse()?;
/// assert_eq!(Tickdelta::from_ticks(24).to_timedelta(rate), Timedelta::MICROSECOND, "at 24 MHz");
/// assert_eq!(Timedelta::SECOND.to_tickdelta(rate), Tickdelta::from_ticks(24_000_000), "and back");
/// # Ok::<(), t2t_core::ParseTickRateError>(())
/// ```
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("{}", Count(hertz.get(), " Hz"))]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy", derive(Immutable, KnownLayout))]
pub struct TickRate {
    /// Ticks a second.
    hertz: NonZeroU64,
    /// Ticks to nanoseconds.
    ticks_to_nanos: Scale,
    /// Nanoseconds to ticks.
    nanos_to_ticks: Scale,
}

impl TickRate {
    /// A tick a nanosecond, the rate the Arm generic timer runs at from Armv8.6 on.
    pub const GIGAHERTZ: Self = Self::from_hertz(1_000_000_000).expect("a billion is no zero");

    /// A counter advancing `hertz` ticks a second, or `None` for zero.
    #[inline]
    #[must_use]
    pub const fn from_hertz(hertz: u64) -> Option<Self> {
        let Some(nonzero) = NonZeroU64::new(hertz) else {
            return None;
        };
        let nanos_per_second = NANOS_PER_SECOND.unsigned_abs();
        Some(Self {
            hertz: nonzero,
            ticks_to_nanos: Scale::new(nanos_per_second, hertz),
            nanos_to_ticks: Scale::new(hertz, nanos_per_second),
        })
    }

    /// Ticks a second.
    #[inline]
    #[must_use]
    pub const fn as_hertz(self) -> u64 {
        self.hertz.get()
    }

    /// `ticks` in nanoseconds, saturating.
    #[inline]
    pub(crate) const fn ticks_to_nanos(self, ticks: i64) -> i64 {
        self.ticks_to_nanos.apply(ticks)
    }

    /// `nanos` in ticks, saturating.
    #[inline]
    pub(crate) const fn nanos_to_ticks(self, nanos: i64) -> i64 {
        self.nanos_to_ticks.apply(nanos)
    }
}

/// Reads what [`Display`](core::fmt::Display) writes: a count of hertz above zero, then ` Hz`.
impl FromStr for TickRate {
    type Err = ParseTickRateError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let hertz = text.strip_suffix(" Hz").and_then(read_count);
        hertz.and_then(Self::from_hertz).ok_or(ParseTickRateError)
    }
}

/// A ratio in fixed point: a count times `factor`, shifted right by `shift`.
///
/// The factor is rounded up, so an exact multiple converts exactly, and any other count to within
/// one unit of its exact quotient.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "zerocopy", derive(Immutable, KnownLayout))]
struct Scale {
    /// The ratio times two to the `shift`, rounded up.
    factor: u64,
    /// The power of two `factor` is scaled by.
    shift: u32,
}

impl Scale {
    /// `numerator / denominator`, with the largest shift whose factor still fits 64 bits, so the
    /// factor carries every bit of precision it can.
    const fn new(numerator: u64, denominator: u64) -> Self {
        let numerator = widen(numerator);
        let denominator = widen(denominator);
        // `numerator << shift` lands near `denominator << 64` here, and one step below fits.
        let mut shift = 64_u32
            .saturating_add(numerator.leading_zeros())
            .saturating_sub(denominator.leading_zeros());
        while shift > 0 && !fits_u64(quotient(numerator, denominator, shift)) {
            shift = shift.saturating_sub(1);
        }
        Self { factor: narrow(quotient(numerator, denominator, shift)), shift }
    }

    /// `count` scaled by the ratio, saturating at the range's ends.
    #[inline]
    const fn apply(self, count: i64) -> i64 {
        // Neither step wraps: a 64-bit product fits 128 bits, and the shift is below 128.
        let magnitude =
            widen(count.unsigned_abs()).wrapping_mul(widen(self.factor)).wrapping_shr(self.shift);
        let magnitude = if fits_u64(magnitude) { narrow(magnitude) } else { u64::MAX };
        if count < 0 {
            0_i64.saturating_sub_unsigned(magnitude)
        } else {
            0_i64.saturating_add_unsigned(magnitude)
        }
    }
}

/// `numerator << shift`, divided by a nonzero `denominator`, rounded up.
const fn quotient(numerator: u128, denominator: u128, shift: u32) -> u128 {
    numerator.wrapping_shl(shift).div_ceil(denominator)
}

/// `value`, widened.
const fn widen(value: u64) -> u128 {
    #[expect(clippy::as_conversions, reason = "widening is lossless; `From` is not const")]
    let widened = value as u128;
    widened
}

/// Whether `value` fits a `u64`.
const fn fits_u64(value: u128) -> bool {
    value <= widen(u64::MAX)
}

/// `value`, which fits a `u64`, narrowed to one.
const fn narrow(value: u128) -> u64 {
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "every caller checks the value fits; `TryFrom` is not const"
    )]
    let narrowed = value as u64;
    narrowed
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;

    use proptest::prelude::{any, prop_assert, proptest};
    use rstest::rstest;

    use crate::{ParseTickRateError, TickRate, Tickdelta, Timedelta};

    /// A rate of `hertz`.
    fn rate(hertz: u64) -> TickRate {
        TickRate::from_hertz(hertz).expect("a nonzero rate")
    }

    /// `count * numerator / denominator`, exactly, truncated toward zero.
    fn exact(count: i64, numerator: u64, denominator: u64) -> i128 {
        i128::from(count)
            .checked_mul(i128::from(numerator))
            .and_then(|product| product.checked_div(i128::from(denominator)))
            .expect("a 64-bit product fits 128 bits, over a nonzero denominator")
    }

    #[rstest]
    #[case::apple_silicon(24_000_000)]
    #[case::armv8_6(1_000_000_000)]
    #[case::a_tsc(3_000_000_000)]
    #[case::a_tsc_from_its_crystal(3_379_200_000)]
    fn a_second_of_ticks_is_a_second(#[case] hertz: u64) {
        let second = Tickdelta::from_ticks(i64::try_from(hertz).expect("a rate below `i64::MAX`"));
        assert_eq!(second.to_timedelta(rate(hertz)), Timedelta::SECOND, "a second of ticks");
        assert_eq!(Timedelta::SECOND.to_tickdelta(rate(hertz)), second, "and back, exactly");
    }

    #[test]
    fn a_backwards_count_converts_backwards() {
        let span = Tickdelta::from_ticks(-24).to_timedelta(rate(24_000_000));
        assert_eq!(span, -Timedelta::MICROSECOND, "ticks to nanoseconds");
        let ticks = (-Timedelta::MICROSECOND).to_tickdelta(rate(3_000_000_000));
        assert_eq!(ticks, Tickdelta::from_ticks(-3_000), "and nanoseconds to ticks");
    }

    #[test]
    fn a_conversion_past_the_range_saturates() {
        assert_eq!(Tickdelta::MAX.to_timedelta(rate(1)), Timedelta::MAX, "forwards");
        assert_eq!(Tickdelta::MIN.to_timedelta(rate(1)), Timedelta::MIN, "backwards");
        assert_eq!(
            Timedelta::MAX.to_tickdelta(rate(u64::MAX)),
            Tickdelta::MAX,
            "the other way about"
        );
    }

    #[test]
    fn a_rate_reads_back_from_its_hertz() {
        assert_eq!(rate(24_000_000).to_string(), "24000000 Hz", "written as its hertz");
        assert_eq!("24000000 Hz".parse(), Ok(rate(24_000_000)), "and read back");
        assert_eq!(TickRate::GIGAHERTZ.as_hertz(), 1_000_000_000, "a gigahertz in hertz");
        assert_eq!(TickRate::from_hertz(0), None, "a counter that never ticks");
    }

    #[test]
    fn a_width_pads_the_rate_with_its_unit() {
        assert_eq!(format!("[{:>8}]", rate(24)), "[   24 Hz]", "to the right");
        assert_eq!(format!("{:?}", rate(24)), "24 Hz", "and debugged as displayed");
    }

    #[rstest]
    #[case::zero("0 Hz")]
    #[case::no_unit("24000000")]
    #[case::a_sign_display_never_writes("+24 Hz")]
    #[case::kilohertz("24 kHz")]
    #[case::past_the_range("18446744073709551616 Hz")]
    fn a_malformed_rate_is_refused(#[case] text: &str) {
        assert_eq!(text.parse::<TickRate>(), Err(ParseTickRateError), "{text:?} is refused");
    }

    proptest! {
        #[test]
        fn a_conversion_is_within_one_unit_of_the_exact_quotient(
            count in any::<i64>(),
            hertz in 1_u64..,
        ) {
            let rate = rate(hertz);
            let nanos = Tickdelta::from_ticks(count).to_timedelta(rate).as_nanos();
            let ticks = Timedelta::from_nanos(count).to_tickdelta(rate).as_ticks();
            for (actual, exact) in [
                (nanos, exact(count, 1_000_000_000, hertz)),
                (ticks, exact(count, hertz, 1_000_000_000)),
            ] {
                let exact = exact.clamp(i128::from(i64::MIN), i128::from(i64::MAX));
                prop_assert!(i128::from(actual).abs_diff(exact) <= 1, "{actual} against {exact}");
            }
        }
    }
}
