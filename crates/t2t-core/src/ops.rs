//! The arithmetic every point and span shares, written once.

use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::consts::{NANOS_PER_MICROSECOND, NANOS_PER_MILLISECOND, NANOS_PER_SECOND};
use crate::{
    BootTime, RawUptime, TaiTimestamp, Tick, Ticks, TimePoint, Timedelta, Timestamp, Uptime,
};

/// The nearest multiple of `|unit|` at or below `value`, or `value` for a zero unit.
#[inline]
const fn floor(value: i64, unit: i64) -> i64 {
    match value.checked_rem_euclid(unit) {
        Some(remainder) => value.saturating_sub(remainder),
        None => value,
    }
}

/// The nearest multiple of `|unit|` at or above `value`, or `value` for a zero unit.
#[inline]
const fn ceil(value: i64, unit: i64) -> i64 {
    match value.checked_rem_euclid(unit) {
        Some(0) | None => value,
        Some(remainder) => {
            value.saturating_sub(remainder).saturating_add_unsigned(unit.unsigned_abs())
        },
    }
}

/// A span's layout, constants, sign, checked arithmetic and saturating operators.
macro_rules! span {
    ($span:ident) => {
        const _: () = assert!(size_of::<$span>() == size_of::<i64>(), "an `i64`, and nothing else");

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

/// A point's layout, constants, checked arithmetic and saturating operators over its span.
macro_rules! point {
    ($point:ident, $span:ident) => {
        const _: () =
            assert!(size_of::<$point>() == size_of::<i64>(), "an `i64`, and nothing else");

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
            fn from_count(count: i64) -> Self {
                Self(count)
            }

            #[inline]
            fn count(self) -> i64 {
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
                Self(micros.saturating_mul(NANOS_PER_MICROSECOND))
            }

            #[doc = concat!("The point `millis` milliseconds after ", $origin, ", saturating.")]
            #[inline]
            #[must_use]
            pub const fn from_millis(millis: i64) -> Self {
                Self(millis.saturating_mul(NANOS_PER_MILLISECOND))
            }

            #[doc = concat!("The point `secs` seconds after ", $origin, ", saturating.")]
            #[inline]
            #[must_use]
            pub const fn from_secs(secs: i64) -> Self {
                Self(secs.saturating_mul(NANOS_PER_SECOND))
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
                self.0.div_euclid(NANOS_PER_MICROSECOND)
            }

            #[doc = concat!("Whole milliseconds since ", $origin, ", rounded down.")]
            #[inline]
            #[must_use]
            pub const fn as_millis(self) -> i64 {
                self.0.div_euclid(NANOS_PER_MILLISECOND)
            }

            #[doc = concat!("Whole seconds since ", $origin, ", rounded down.")]
            #[inline]
            #[must_use]
            pub const fn as_secs(self) -> i64 {
                self.0.div_euclid(NANOS_PER_SECOND)
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
        let (after, before) = (Timestamp::from_nanos(10_500), Timestamp::from_nanos(-10_500));
        assert_eq!(after.floor(unit), Timestamp::from_nanos(10_000), "down, after the epoch");
        assert_eq!(after.ceil(unit), Timestamp::from_nanos(11_000), "and up");
        assert_eq!(before.floor(unit), Timestamp::from_nanos(-11_000), "down, before it");
        assert_eq!(before.ceil(unit), Timestamp::from_nanos(-10_000), "and up");
        let multiple = Timestamp::from_nanos(11_000);
        assert_eq!(multiple.ceil(unit), multiple, "a multiple stays where it is");
    }

    #[test]
    fn floor_and_ceil_take_the_units_magnitude_and_pass_over_a_zero_unit() {
        let instant = Timestamp::from_nanos(10_500);
        let backwards = -Timedelta::MICROSECOND;
        assert_eq!(instant.floor(backwards), Timestamp::from_nanos(10_000), "a backwards unit");
        assert_eq!(instant.floor(Timedelta::ZERO), instant, "no unit");
        let longest = Timestamp::from_nanos(-5).ceil(Timedelta::MIN);
        assert_eq!(longest, Timestamp::UNIX_EPOCH, "the longest unit, whose magnitude is no `i64`");
    }

    #[test]
    fn point_arithmetic_saturates_and_its_checked_twin_refuses() {
        let span = Timedelta::NANOSECOND;
        assert_eq!(Timestamp::MAX + span, Timestamp::MAX, "a point past the end saturates");
        assert_eq!(Timestamp::MIN - span, Timestamp::MIN, "at both ends");
        assert_eq!(Timestamp::MAX.checked_add(span), None, "and its checked twin refuses");
        assert_eq!(Timestamp::MIN.checked_sub(span), None, "at both ends");
        let widest = Timestamp::MAX - Timestamp::MIN;
        assert_eq!(widest, Timedelta::MAX, "a span past its range saturates");
        assert_eq!(Timestamp::MAX.checked_since(Timestamp::MIN), None, "or is refused");
        let (start, end) = (Tick::from_ticks(1_000), Tick::from_ticks(1_025));
        assert_eq!(end - start, Ticks::from_ticks(25), "a later reading minus an earlier");
        assert_eq!(start - end, Ticks::from_ticks(-25), "and the other way about");
    }

    #[test]
    fn span_arithmetic_saturates_and_its_checked_twin_refuses() {
        let ten = Timedelta::from_nanos(10);
        assert_eq!(-ten + ten * 3, Timedelta::from_nanos(20), "negation, sum and product");
        assert_eq!(3 * ten, ten * 3, "a factor on either side");
        assert_eq!([ten, ten, ten].iter().sum::<Timedelta>(), ten * 3, "and a sum of many");
        assert_eq!(Timedelta::MAX + ten, Timedelta::MAX, "past the end saturates");
        assert_eq!(-Timedelta::MIN, Timedelta::MAX, "so does the negation with no twin");
        assert_eq!(Timedelta::MIN.abs(), Timedelta::MAX, "and the magnitude");
        assert_eq!(Timedelta::MIN.checked_abs(), None, "which the checked twins refuse");
        assert_eq!(Timedelta::MIN.checked_neg(), None, "both");
        assert_eq!(ten.checked_div(0), None, "no parts");
        assert_eq!(Timedelta::MIN.checked_div(-1), None, "a quotient past the range");
        let part = Timedelta::from_nanos(-7).checked_div(2);
        assert_eq!(part, Some(Timedelta::from_nanos(-3)), "truncated toward zero");
    }

    #[test]
    fn nanosecond_accessors_round_down_before_the_origin() {
        assert_eq!(Timestamp::from_nanos(-1).as_secs(), -1, "the second it falls in");
        assert_eq!(Uptime::from_secs(90).as_millis(), 90_000, "a coarser unit, counted finer");
        assert_eq!(
            Timestamp::from_secs(i64::MAX),
            Timestamp::MAX,
            "a count past the range saturates"
        );
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
            prop_assert_eq!(multiple.as_nanos().rem_euclid(unit), 0, "a multiple of the unit");
            prop_assert!(
                distance.is_some_and(|distance| (0..unit).contains(&distance)),
                "less than a unit below"
            );
        }
    }
}
