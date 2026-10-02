//! A counter's rate, and the conversion between its ticks and nanoseconds.

use core::fmt;
use core::num::NonZeroU64;

use crate::timedelta::NANOS_PER_SEC;
use crate::{Ticks, Timedelta};

/// How many ticks a counter advances in a second: what turns [`Ticks`] into a [`Timedelta`] and
/// back.
///
/// A conversion is a multiply and a shift, with factors [`new`](Self::new) works out once: no
/// division. An exact multiple converts exactly, any other count to within one unit of its exact
/// quotient, and the range's ends saturate.
///
/// # Examples
/// ```
/// use t2t_core::{TickRate, Ticks, Timedelta};
///
/// let rate = TickRate::new(24_000_000).expect("a nonzero rate");
/// assert_eq!(rate.timedelta(Ticks::new(24)), Timedelta::MICROSECOND, "24 ticks at 24 MHz");
/// assert_eq!(rate.ticks(Timedelta::SECOND), Ticks::new(24_000_000), "and back");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "zerocopy", derive(zerocopy::Immutable, zerocopy::KnownLayout))]
pub struct TickRate {
    /// Ticks a second.
    rate: NonZeroU64,
    /// Ticks to nanoseconds.
    ticks_to_nanos: Scale,
    /// Nanoseconds to ticks.
    nanos_to_ticks: Scale,
}

impl TickRate {
    /// A tick a nanosecond, the rate the Arm generic timer runs at from Armv8.6 on.
    pub const GIGAHERTZ: Self = Self::new(1_000_000_000).unwrap();

    /// A counter advancing `ticks_per_second` times a second, or `None` for zero.
    #[must_use]
    pub const fn new(ticks_per_second: u64) -> Option<Self> {
        let Some(rate) = NonZeroU64::new(ticks_per_second) else {
            return None;
        };
        let nanos_per_second = NANOS_PER_SEC.unsigned_abs();
        Some(Self {
            rate,
            ticks_to_nanos: Scale::new(nanos_per_second, ticks_per_second),
            nanos_to_ticks: Scale::new(ticks_per_second, nanos_per_second),
        })
    }

    /// Ticks a second.
    #[inline]
    #[must_use]
    pub const fn get(self) -> u64 {
        self.rate.get()
    }

    /// How long `ticks` lasts at this rate.
    #[inline]
    #[must_use]
    pub const fn timedelta(self, ticks: Ticks) -> Timedelta {
        Timedelta(self.ticks_to_nanos.apply(ticks.0))
    }

    /// How many ticks `span` lasts at this rate.
    #[inline]
    #[must_use]
    pub const fn ticks(self, span: Timedelta) -> Ticks {
        Ticks(self.nanos_to_ticks.apply(span.0))
    }
}

/// The rate and its unit: `24000000 Hz`.
impl fmt::Display for TickRate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} Hz", self.rate)
    }
}

/// As [`Display`](fmt::Display) writes it.
impl fmt::Debug for TickRate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

/// A ratio in fixed point: a count times `factor`, shifted right by `shift`.
///
/// The factor is rounded up, so an exact multiple converts exactly, and any other count to within
/// one unit of its exact quotient.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "zerocopy", derive(zerocopy::Immutable, zerocopy::KnownLayout))]
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
    use alloc::string::ToString as _;

    use proptest::prelude::{any, prop_assert, proptest};
    use rstest::rstest;

    use crate::{TickRate, Ticks, Timedelta};

    /// A rate of `ticks_per_second`.
    fn rate(ticks_per_second: u64) -> TickRate {
        TickRate::new(ticks_per_second).expect("a nonzero rate")
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
    fn a_second_of_ticks_is_a_second(#[case] ticks_per_second: u64) {
        let second = Ticks::new(i64::try_from(ticks_per_second).expect("a rate below `i64::MAX`"));
        assert_eq!(rate(ticks_per_second).timedelta(second), Timedelta::SECOND);
        assert_eq!(rate(ticks_per_second).ticks(Timedelta::SECOND), second);
    }

    #[test]
    fn a_backwards_count_converts_backwards() {
        assert_eq!(rate(24_000_000).timedelta(Ticks::new(-24)), -Timedelta::MICROSECOND);
        assert_eq!(rate(3_000_000_000).ticks(-Timedelta::MICROSECOND), Ticks::new(-3_000));
    }

    #[test]
    fn a_conversion_past_the_range_saturates() {
        assert_eq!(rate(1).timedelta(Ticks::MAX), Timedelta::MAX);
        assert_eq!(rate(1).timedelta(Ticks::MIN), Timedelta::MIN);
        assert_eq!(rate(u64::MAX).ticks(Timedelta::MAX), Ticks::MAX);
    }

    #[test]
    fn a_rate_of_zero_is_refused_and_any_other_writes_its_unit() {
        assert_eq!(TickRate::new(0), None);
        assert_eq!(rate(24_000_000).to_string(), "24000000 Hz");
        assert_eq!(TickRate::GIGAHERTZ.get(), 1_000_000_000);
    }

    proptest! {
        #[test]
        fn a_conversion_is_within_one_unit_of_the_exact_quotient(
            count in any::<i64>(),
            ticks_per_second in 1_u64..=20_000_000_000,
        ) {
            let rate = rate(ticks_per_second);
            let nanos = rate.timedelta(Ticks::new(count)).as_nanos();
            let ticks = rate.ticks(Timedelta::from_nanos(count)).get();
            for (actual, exact) in [
                (nanos, exact(count, 1_000_000_000, ticks_per_second)),
                (ticks, exact(count, ticks_per_second, 1_000_000_000)),
            ] {
                let exact = exact.clamp(i128::from(i64::MIN), i128::from(i64::MAX));
                prop_assert!(i128::from(actual).abs_diff(exact) <= 1, "{actual} against {exact}");
            }
        }
    }
}
