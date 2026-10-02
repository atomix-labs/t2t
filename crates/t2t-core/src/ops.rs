//! The arithmetic every point and span shares, written once.

use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::timedelta::{NANOS_PER_MICRO, NANOS_PER_MILLI, NANOS_PER_SEC};
use crate::{
    BootTime, RawUptime, TaiTimestamp, Tick, Ticks, TimePoint, Timedelta, Timestamp, Uptime,
};

/// The nearest multiple of `|unit|` at or below `value`, or `value` for a zero unit.
const fn floor(value: i64, unit: i64) -> i64 {
    match value.checked_rem_euclid(unit) {
        Some(remainder) => value.saturating_sub(remainder),
        None => value,
    }
}

/// The nearest multiple of `|unit|` at or above `value`, or `value` for a zero unit.
const fn ceil(value: i64, unit: i64) -> i64 {
    match value.checked_rem_euclid(unit) {
        Some(0) | None => value,
        Some(remainder) => {
            value.saturating_sub(remainder).saturating_add_unsigned(unit.unsigned_abs())
        },
    }
}

/// `value` divided by a positive `divisor`, rounded down.
const fn floor_div(value: i64, divisor: i64) -> i64 {
    match value.checked_div_euclid(divisor) {
        Some(quotient) => quotient,
        None => 0,
    }
}

/// A span's constants, sign, checked arithmetic and saturating operators.
macro_rules! span {
    ($span:ident) => {
        impl $span {
            /// No time at all.
            pub const ZERO: Self = Self(0);
            /// The longest span backwards.
            pub const MIN: Self = Self(i64::MIN);
            /// The longest span.
            pub const MAX: Self = Self(i64::MAX);

            /// Whether the span is zero.
            #[inline]
            #[must_use]
            pub const fn is_zero(self) -> bool {
                self.0 == 0
            }

            /// Whether the span runs backwards.
            #[inline]
            #[must_use]
            pub const fn is_negative(self) -> bool {
                self.0 < 0
            }

            /// Whether the span runs forwards.
            #[inline]
            #[must_use]
            pub const fn is_positive(self) -> bool {
                self.0 > 0
            }

            /// The span forwards, [`MAX`](Self::MAX) for [`MIN`](Self::MIN).
            #[inline]
            #[must_use]
            pub const fn abs(self) -> Self {
                Self(self.0.saturating_abs())
            }

            /// The sum, or `None` past the range.
            #[inline]
            #[must_use]
            pub const fn checked_add(self, other: Self) -> Option<Self> {
                match self.0.checked_add(other.0) {
                    Some(sum) => Some(Self(sum)),
                    None => None,
                }
            }

            /// The difference, or `None` past the range.
            #[inline]
            #[must_use]
            pub const fn checked_sub(self, other: Self) -> Option<Self> {
                match self.0.checked_sub(other.0) {
                    Some(difference) => Some(Self(difference)),
                    None => None,
                }
            }

            /// The span `factor` times over, or `None` past the range.
            #[inline]
            #[must_use]
            pub const fn checked_mul(self, factor: i64) -> Option<Self> {
                match self.0.checked_mul(factor) {
                    Some(product) => Some(Self(product)),
                    None => None,
                }
            }

            /// One of `parts` equal parts, truncated toward zero, or `None` for no parts, or for
            /// [`MIN`](Self::MIN) in `-1`.
            #[inline]
            #[must_use]
            pub const fn checked_div(self, parts: i64) -> Option<Self> {
                match self.0.checked_div(parts) {
                    Some(part) => Some(Self(part)),
                    None => None,
                }
            }

            /// The span backwards, or `None` for [`MIN`](Self::MIN).
            #[inline]
            #[must_use]
            pub const fn checked_neg(self) -> Option<Self> {
                match self.0.checked_neg() {
                    Some(negation) => Some(Self(negation)),
                    None => None,
                }
            }

            /// The span forwards, or `None` for [`MIN`](Self::MIN).
            #[inline]
            #[must_use]
            pub const fn checked_abs(self) -> Option<Self> {
                match self.0.checked_abs() {
                    Some(magnitude) => Some(Self(magnitude)),
                    None => None,
                }
            }

            /// The nearest multiple of `|unit|` at or below the span, or the span for a zero unit.
            #[inline]
            #[must_use]
            pub const fn floor(self, unit: Self) -> Self {
                Self(floor(self.0, unit.0))
            }

            /// The nearest multiple of `|unit|` at or above the span, or the span for a zero unit.
            #[inline]
            #[must_use]
            pub const fn ceil(self, unit: Self) -> Self {
                Self(ceil(self.0, unit.0))
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl Add for $span {
            type Output = Self;

            #[inline]
            fn add(self, other: Self) -> Self {
                Self(self.0.saturating_add(other.0))
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl AddAssign for $span {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = *self + other;
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl Sub for $span {
            type Output = Self;

            #[inline]
            fn sub(self, other: Self) -> Self {
                Self(self.0.saturating_sub(other.0))
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl SubAssign for $span {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other;
            }
        }

        /// [`MAX`](Self::MAX) for [`MIN`](Self::MIN).
        impl Neg for $span {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                Self(self.0.saturating_neg())
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl Mul<i64> for $span {
            type Output = Self;

            #[inline]
            fn mul(self, factor: i64) -> Self {
                Self(self.0.saturating_mul(factor))
            }
        }

        /// Saturates at the span's range.
        impl Mul<$span> for i64 {
            type Output = $span;

            #[inline]
            fn mul(self, span: $span) -> $span {
                span * self
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl MulAssign<i64> for $span {
            #[inline]
            fn mul_assign(&mut self, factor: i64) {
                *self = *self * factor;
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl Sum for $span {
            #[inline]
            fn sum<I: Iterator<Item = Self>>(spans: I) -> Self {
                spans.fold(Self::ZERO, Add::add)
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl<'a> Sum<&'a Self> for $span {
            #[inline]
            fn sum<I: Iterator<Item = &'a Self>>(spans: I) -> Self {
                spans.copied().sum()
            }
        }
    };
}

/// A point's constants, checked arithmetic and saturating operators over its span.
macro_rules! point {
    ($point:ident, $span:ident) => {
        impl $point {
            /// The earliest point the count holds.
            pub const MIN: Self = Self(i64::MIN);
            /// The latest point the count holds.
            pub const MAX: Self = Self(i64::MAX);

            /// The point `span` later, or `None` past the range.
            #[inline]
            #[must_use]
            pub const fn checked_add(self, span: $span) -> Option<Self> {
                match self.0.checked_add(span.0) {
                    Some(point) => Some(Self(point)),
                    None => None,
                }
            }

            /// The point `span` earlier, or `None` past the range.
            #[inline]
            #[must_use]
            pub const fn checked_sub(self, span: $span) -> Option<Self> {
                match self.0.checked_sub(span.0) {
                    Some(point) => Some(Self(point)),
                    None => None,
                }
            }

            /// The span since `start`, or `None` past the span's range.
            #[inline]
            #[must_use]
            pub const fn checked_since(self, start: Self) -> Option<$span> {
                match self.0.checked_sub(start.0) {
                    Some(span) => Some($span(span)),
                    None => None,
                }
            }

            /// The nearest multiple of `|unit|` at or below the point, or the point for a zero
            /// unit.
            #[inline]
            #[must_use]
            pub const fn floor(self, unit: $span) -> Self {
                Self(floor(self.0, unit.0))
            }

            /// The nearest multiple of `|unit|` at or above the point, or the point for a zero
            /// unit.
            #[inline]
            #[must_use]
            pub const fn ceil(self, unit: $span) -> Self {
                Self(ceil(self.0, unit.0))
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl Add<$span> for $point {
            type Output = Self;

            #[inline]
            fn add(self, span: $span) -> Self {
                Self(self.0.saturating_add(span.0))
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl AddAssign<$span> for $point {
            #[inline]
            fn add_assign(&mut self, span: $span) {
                *self = *self + span;
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl Sub<$span> for $point {
            type Output = Self;

            #[inline]
            fn sub(self, span: $span) -> Self {
                Self(self.0.saturating_sub(span.0))
            }
        }

        /// Saturates at [`MIN`](Self::MIN) and [`MAX`](Self::MAX).
        impl SubAssign<$span> for $point {
            #[inline]
            fn sub_assign(&mut self, span: $span) {
                *self = *self - span;
            }
        }

        /// The span since the other point, saturating at the span's range.
        impl Sub for $point {
            type Output = $span;

            #[inline]
            fn sub(self, start: Self) -> $span {
                $span(self.0.saturating_sub(start.0))
            }
        }

        impl TimePoint for $point {
            type Span = $span;

            #[inline]
            fn from_i64(value: i64) -> Self {
                Self(value)
            }

            #[inline]
            fn to_i64(self) -> i64 {
                self.0
            }
        }
    };
}

/// A nanosecond point's constructors and accessors in each unit, counted from `$origin`.
macro_rules! nanosecond_units {
    ($point:ident, $origin:literal) => {
        impl $point {
            #[doc = concat!("The point `nanos` nanoseconds after ", $origin, ".")]
            #[inline]
            #[must_use]
            pub const fn from_nanos(nanos: i64) -> Self {
                Self(nanos)
            }

            #[doc = concat!("The point `micros` microseconds after ", $origin, ", saturating.")]
            #[inline]
            #[must_use]
            pub const fn from_micros(micros: i64) -> Self {
                Self(micros.saturating_mul(NANOS_PER_MICRO))
            }

            #[doc = concat!("The point `millis` milliseconds after ", $origin, ", saturating.")]
            #[inline]
            #[must_use]
            pub const fn from_millis(millis: i64) -> Self {
                Self(millis.saturating_mul(NANOS_PER_MILLI))
            }

            #[doc = concat!("The point `secs` seconds after ", $origin, ", saturating.")]
            #[inline]
            #[must_use]
            pub const fn from_secs(secs: i64) -> Self {
                Self(secs.saturating_mul(NANOS_PER_SEC))
            }

            #[doc = concat!("Nanoseconds since ", $origin, ".")]
            #[inline]
            #[must_use]
            pub const fn as_nanos(self) -> i64 {
                self.0
            }

            #[doc = concat!("Whole microseconds since ", $origin, ", rounded down.")]
            #[inline]
            #[must_use]
            pub const fn as_micros(self) -> i64 {
                floor_div(self.0, NANOS_PER_MICRO)
            }

            #[doc = concat!("Whole milliseconds since ", $origin, ", rounded down.")]
            #[inline]
            #[must_use]
            pub const fn as_millis(self) -> i64 {
                floor_div(self.0, NANOS_PER_MILLI)
            }

            #[doc = concat!("Whole seconds since ", $origin, ", rounded down.")]
            #[inline]
            #[must_use]
            pub const fn as_secs(self) -> i64 {
                floor_div(self.0, NANOS_PER_SEC)
            }
        }
    };
}

span!(Timedelta);
span!(Ticks);

point!(Timestamp, Timedelta);
point!(TaiTimestamp, Timedelta);
point!(Uptime, Timedelta);
point!(RawUptime, Timedelta);
point!(BootTime, Timedelta);
point!(Tick, Ticks);

nanosecond_units!(Timestamp, "the Unix epoch");
nanosecond_units!(TaiTimestamp, "1970-01-01T00:00:00 TAI");
nanosecond_units!(Uptime, "the monotonic clock's origin");
nanosecond_units!(RawUptime, "the raw monotonic clock's origin");
nanosecond_units!(BootTime, "boot");

#[cfg(test)]
mod tests {
    use core::hint::black_box;

    use proptest::prelude::{prop_assert, prop_assert_eq, prop_assume, proptest};

    use crate::{Tick, Ticks, Timedelta, Timestamp, Uptime};

    #[test]
    fn floor_and_ceil_tile_the_line_on_both_sides_of_zero() {
        let unit = Timedelta::MICROSECOND;
        assert_eq!(Timestamp::from_nanos(10_500).floor(unit), Timestamp::from_nanos(10_000));
        assert_eq!(Timestamp::from_nanos(10_500).ceil(unit), Timestamp::from_nanos(11_000));
        assert_eq!(Timestamp::from_nanos(-10_500).floor(unit), Timestamp::from_nanos(-11_000));
        assert_eq!(Timestamp::from_nanos(-10_500).ceil(unit), Timestamp::from_nanos(-10_000));
        assert_eq!(Timestamp::from_nanos(11_000).ceil(unit), Timestamp::from_nanos(11_000));
    }

    #[test]
    fn floor_and_ceil_take_the_units_magnitude_and_pass_over_a_zero_unit() {
        let instant = Timestamp::from_nanos(10_500);
        assert_eq!(instant.floor(-Timedelta::MICROSECOND), Timestamp::from_nanos(10_000));
        assert_eq!(instant.floor(Timedelta::ZERO), instant);
        assert_eq!(Timestamp::from_nanos(-5).ceil(Timedelta::MIN), Timestamp::UNIX_EPOCH);
    }

    #[test]
    fn point_arithmetic_saturates_and_its_checked_twin_refuses() {
        let span = Timedelta::NANOSECOND;
        assert_eq!(Timestamp::MAX + span, Timestamp::MAX);
        assert_eq!(Timestamp::MIN - span, Timestamp::MIN);
        assert_eq!(Timestamp::MAX.checked_add(span), None);
        assert_eq!(Timestamp::MIN.checked_sub(span), None);
        assert_eq!(Timestamp::MAX - Timestamp::MIN, Timedelta::MAX);
        assert_eq!(Timestamp::MAX.checked_since(Timestamp::MIN), None);
        assert_eq!(Tick::new(1_025) - Tick::new(1_000), Ticks::new(25));
        assert_eq!(Tick::new(1_000) - Tick::new(1_025), Ticks::new(-25));
    }

    #[test]
    fn span_arithmetic_saturates_and_its_checked_twin_refuses() {
        let ten = Timedelta::from_nanos(10);
        assert_eq!(-ten + ten * 3, Timedelta::from_nanos(20));
        assert_eq!(3 * ten, ten * 3);
        assert_eq!(Timedelta::MAX + ten, Timedelta::MAX);
        assert_eq!(-Timedelta::MIN, Timedelta::MAX);
        assert_eq!(Timedelta::MIN.abs(), Timedelta::MAX);
        assert_eq!(Timedelta::MIN.checked_abs(), None);
        assert_eq!(Timedelta::MIN.checked_neg(), None);
        assert_eq!(ten.checked_div(0), None);
        assert_eq!(Timedelta::MIN.checked_div(-1), None);
        assert_eq!(Timedelta::from_nanos(-7).checked_div(2), Some(Timedelta::from_nanos(-3)));
        assert_eq!([ten, ten, ten].iter().sum::<Timedelta>(), ten * 3);
    }

    #[test]
    fn nanosecond_accessors_round_down_before_the_origin() {
        assert_eq!(Timestamp::from_nanos(-1).as_secs(), -1, "the second it falls in");
        assert_eq!(Uptime::from_secs(90).as_millis(), 90_000);
        assert_eq!(Timestamp::from_secs(i64::MAX), Timestamp::MAX, "saturating");
    }

    proptest! {
        #[test]
        fn every_operator_saturates_rather_than_panics(left: i64, right: i64, factor: i64) {
            let (point, span) = (Timestamp::from_nanos(left), Timedelta::from_nanos(right));
            black_box((point + span, point - span, point - Timestamp::from_nanos(right)));
            black_box((span + span, span - span, -span, span * factor, span.abs()));
            black_box((point.floor(span), point.ceil(span), span.floor(span), span.ceil(span)));
        }

        #[test]
        fn a_floor_is_the_multiple_at_or_below(value: i64, unit in 1_i64..1_000_000_000_000) {
            let multiple = Timestamp::from_nanos(value).floor(Timedelta::from_nanos(unit));
            prop_assume!(multiple != Timestamp::MIN, "saturated");
            let distance = value.checked_sub(multiple.as_nanos());
            prop_assert_eq!(multiple.as_nanos().rem_euclid(unit), 0);
            prop_assert!(distance.is_some_and(|distance| (0..unit).contains(&distance)));
        }
    }
}
