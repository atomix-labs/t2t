//! The trait every point on a timeline shares.

use core::ops::{Add, Sub};

/// A point on one timeline, carried as a count of its unit since the timeline's origin, with the
/// span between two of them.
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
/// assert_eq!(Timestamp::from_count(stamp.count()), stamp, "and its count round trips");
/// ```
pub trait TimePoint:
    Copy + Ord + Add<Self::Span, Output = Self> + Sub<Output = Self::Span>
{
    /// The span between two points.
    type Span: Copy + Ord;

    /// The point `count` units after the origin.
    #[must_use]
    fn from_count(count: i64) -> Self;

    /// The point's count of units since the origin.
    #[must_use]
    fn count(self) -> i64;
}
