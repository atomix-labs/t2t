//! A value and the stamp it was captured with.

use core::ops::{Deref, DerefMut};

use crate::{TimePoint, Timestamp};

/// A value `T` and the stamp `S` its writer captured it with.
///
/// The stamp comes from the writer: by the time a reader holds the value the moment has gone, and
/// a clock read on the reading side would time the reader. The stamp comes first, so the derived
/// order is chronological, and the value is reached through [`Deref`].
///
/// # Examples
/// ```
/// use t2t_core::{Timed, Timedelta, Timestamp};
///
/// let captured = Timestamp::from_secs(1_700_000_000);
/// let quote = Timed::new(captured, 101_u64);
///
/// assert_eq!(*quote, 101, "the value, through `Deref`");
/// let now = captured + Timedelta::from_millis(5);
/// assert_eq!(quote.elapsed(now), Timedelta::from_millis(5), "its age");
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(
        zerocopy::FromBytes,
        zerocopy::IntoBytes,
        zerocopy::Immutable,
        zerocopy::KnownLayout,
        zerocopy::Unaligned
    )
)]
pub struct Timed<T, S = Timestamp> {
    /// When the value was captured.
    pub stamp: S,
    /// What was captured.
    pub value: T,
}

const _: () = assert!(size_of::<Timed<u64>>() == 16, "a stamp and a word, with no padding");

impl<T, S> Timed<T, S> {
    /// `value`, captured at `stamp`.
    #[inline]
    #[must_use]
    pub const fn new(stamp: S, value: T) -> Self {
        Self { stamp, value }
    }

    /// The value, without its stamp.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// The stamp and the value.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (S, T) {
        (self.stamp, self.value)
    }

    /// The same stamp over a value made from this one.
    #[inline]
    #[must_use]
    pub fn map<U, F: FnOnce(T) -> U>(self, transform: F) -> Timed<U, S> {
        Timed { stamp: self.stamp, value: transform(self.value) }
    }

    /// The same value under a stamp made from this one.
    #[inline]
    #[must_use]
    pub fn map_stamp<R, F: FnOnce(S) -> R>(self, transform: F) -> Timed<T, R> {
        Timed { stamp: transform(self.stamp), value: self.value }
    }
}

impl<T, S: Copy> Timed<T, S> {
    /// The same stamp over a borrow of the value.
    #[inline]
    #[must_use]
    pub const fn as_ref(&self) -> Timed<&T, S> {
        Timed { stamp: self.stamp, value: &self.value }
    }

    /// The same stamp over a mutable borrow of the value.
    #[inline]
    #[must_use]
    pub const fn as_mut(&mut self) -> Timed<&mut T, S> {
        Timed { stamp: self.stamp, value: &mut self.value }
    }
}

impl<T, S: TimePoint> Timed<T, S> {
    /// The span from capture to `now`, negative for a stamp still ahead.
    #[inline]
    #[must_use]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "the point's own subtraction, which saturates for t2t's"
    )]
    pub fn elapsed(&self, now: S) -> S::Span {
        now - self.stamp
    }
}

impl<T, S> Deref for Timed<T, S> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T, S> DerefMut for Timed<T, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T, S> From<(S, T)> for Timed<T, S> {
    #[inline]
    fn from((stamp, value): (S, T)) -> Self {
        Self::new(stamp, value)
    }
}

impl<T, S> From<Timed<T, S>> for (S, T) {
    #[inline]
    fn from(timed: Timed<T, S>) -> Self {
        timed.into_parts()
    }
}

#[cfg(test)]
mod tests {
    use core::mem::offset_of;

    use crate::{Tick, Ticks, Timed, Timedelta, Timestamp, Uptime};

    #[test]
    fn the_stamp_leads() {
        assert_eq!(offset_of!(Timed<u64>, stamp), 0);
    }

    #[test]
    fn the_order_is_chronological_whatever_the_value() {
        let first_quote = Timed::new(Timestamp::from_secs(1), 99_u64);
        let second_quote = Timed::new(Timestamp::from_secs(2), 0_u64);
        assert!(first_quote < second_quote, "the stamp decides");
    }

    #[test]
    fn elapsed_is_the_stamps_own_span() {
        assert_eq!(
            Timed::new(Timestamp::from_secs(10), ()).elapsed(Timestamp::from_secs(13)),
            Timedelta::from_secs(3)
        );
        assert_eq!(
            Timed::new(Uptime::from_secs(10), ()).elapsed(Uptime::from_secs(9)),
            -Timedelta::SECOND
        );
        assert_eq!(Timed::new(Tick::new(1_000), ()).elapsed(Tick::new(1_025)), Ticks::new(25));
    }

    #[test]
    fn mapping_keeps_the_other_half() {
        let timed = Timed::new(Timestamp::from_secs(7), 21_u64);
        assert_eq!(timed.map(|value| value * 2), Timed::new(Timestamp::from_secs(7), 42));
        assert_eq!(timed.map_stamp(Timestamp::as_secs), Timed::new(7, 21));
        assert_eq!(timed.into_parts(), (Timestamp::from_secs(7), 21));
        assert_eq!(Timed::from(timed.into_parts()), timed);
    }

    #[test]
    fn a_borrow_keeps_the_stamp() {
        let mut timed = Timed::new(Timestamp::from_secs(5), [1_u8, 2, 3]);
        assert_eq!(timed.len(), 3, "the value's own methods, through `Deref`");
        timed.as_mut().value[0] = 9;
        assert_eq!(timed.as_ref().value, &[9, 2, 3]);
    }
}
