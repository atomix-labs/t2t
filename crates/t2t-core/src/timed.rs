//! A value and the stamp it was captured with.

use derive_more::{Deref, DerefMut, From, Into};
#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::{TimePoint, Timestamp};

/// A value `T` and the stamp `S` its writer captured it with.
///
/// The stamp comes from the writer: by the time a reader holds the value the moment has gone, and
/// a clock read on the reading side would time the reader. The stamp comes first, so the derived
/// order is chronological, and the value is reached through [`Deref`](core::ops::Deref).
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deref, DerefMut, From, Into,
)]
#[cfg_attr(feature = "zerocopy", derive(FromBytes, Immutable, KnownLayout))]
pub struct Timed<T, S = Timestamp> {
    /// When the value was captured.
    pub stamp: S,
    /// What was captured.
    #[deref]
    #[deref_mut]
    pub value: T,
}

const _: () = assert!(size_of::<Timed<u64>>() == 16, "a stamp and a word, with no padding");

// What `zerocopy` gives a stamped value: read from bytes, never written to them, since its
// `IntoBytes` derive takes a generic struct only where every field is `Unaligned`, and no stamp is.
#[cfg(feature = "zerocopy")]
const _: () = {
    /// Compiles only for a type read from any bytes.
    const fn is_from_bytes<T: FromBytes>() {}
    is_from_bytes::<Timed<u64>>();
};

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

#[cfg(test)]
mod tests {
    use core::mem::offset_of;

    use crate::{Tickdelta, Tickstamp, Timed, Timedelta, Timestamp, Uptime};

    #[test]
    fn the_stamp_leads() {
        assert_eq!(offset_of!(Timed<u64>, stamp), 0, "the stamp at the front, for its order");
    }

    #[test]
    fn the_order_is_chronological_whatever_the_value() {
        let first_quote = Timed::new(Timestamp::from_secs(1), 99_u64);
        let second_quote = Timed::new(Timestamp::from_secs(2), 0_u64);
        assert!(first_quote < second_quote, "the stamp decides");
    }

    #[test]
    fn elapsed_is_the_span_on_the_stamps_own_timeline() {
        let wall = Timed::new(Timestamp::from_secs(10), ()).elapsed(Timestamp::from_secs(13));
        assert_eq!(wall, Timedelta::from_secs(3), "on the wall clock");
        let ahead = Timed::new(Uptime::from_secs(10), ()).elapsed(Uptime::from_secs(9));
        assert_eq!(ahead, -Timedelta::SECOND, "negative for a stamp still ahead");
        let counted =
            Timed::new(Tickstamp::from_ticks(1_000), ()).elapsed(Tickstamp::from_ticks(1_025));
        assert_eq!(counted, Tickdelta::from_ticks(25), "and in ticks on a counter");
    }

    #[test]
    fn mapping_keeps_the_other_half() {
        let timed = Timed::new(Timestamp::from_secs(7), 21_u64);
        let doubled = Timed::new(Timestamp::from_secs(7), 42);
        assert_eq!(timed.map(|value| value * 2), doubled, "a new value, the same stamp");
        assert_eq!(timed.map_stamp(Timestamp::as_secs), Timed::new(7, 21), "and the other way");
        let parts = (Timestamp::from_secs(7), 21);
        assert_eq!(timed.into_parts(), parts, "the stamp and the value");
        assert_eq!(Timed::from(parts), timed, "and back");
        assert_eq!(<(Timestamp, u64)>::from(timed), parts, "through `From` both ways");
    }

    #[test]
    fn a_borrow_keeps_the_stamp() {
        let mut timed = Timed::new(Timestamp::from_secs(5), [1_u8, 2, 3]);
        assert_eq!(timed.len(), 3, "the value's own methods, through `Deref`");
        timed.as_mut().value[0] = 9;
        assert_eq!(timed.as_ref().value, &[9, 2, 3], "a borrow under the same stamp");
    }
}
