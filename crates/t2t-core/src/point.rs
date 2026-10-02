//! The trait every point on a timeline shares.

use core::ops::{Add, Sub};

/// A point on one timeline, carried as an `i64`, with the span between two of them.
///
/// Every point here implements it, and so may a type of your own that wraps one, to work with code
/// generic over points.
///
/// # Examples
/// ```
/// use t2t_core::{TimePoint, Timedelta, Timestamp};
///
/// fn age<P: TimePoint>(stamp: P, now: P) -> P::Span {
///     now - stamp
/// }
///
/// let stamp = Timestamp::from_secs(10);
/// assert_eq!(age(stamp, stamp + Timedelta::SECOND), Timedelta::SECOND, "any point's age");
/// assert_eq!(Timestamp::from_i64(stamp.to_i64()), stamp, "and its count round trips");
/// ```
pub trait TimePoint:
    Copy + Ord + Add<Self::Span, Output = Self> + Sub<Output = Self::Span>
{
    /// The span between two points.
    type Span: Copy + Ord;

    /// The point whose count is `value`.
    #[must_use]
    fn from_i64(value: i64) -> Self;

    /// The point's count.
    #[must_use]
    fn to_i64(self) -> i64;
}
