//! A hardware counter's readings, and the span between two.

/// One reading of a free-running hardware counter.
///
/// The counter runs from an origin the hardware chose, so only the difference of two readings
/// means anything, and only with the counter's [`TickRate`](crate::TickRate). Every core and
/// process on a machine reads one counter, so a reading one process took, another may subtract
/// from.
///
/// # Examples
/// ```
/// use t2t_core::{Tick, TickRate, Ticks, Timedelta};
///
/// let start = Tick::new(1_000);
/// let end = start + Ticks::new(24);
///
/// assert_eq!(end - start, Ticks::new(24), "a reading minus a reading");
/// let rate = TickRate::new(24_000_000).expect("a nonzero rate");
/// assert_eq!(rate.timedelta(end - start), Timedelta::MICROSECOND, "worth a span at a rate");
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct Tick(pub(crate) i64);

const _: () = assert!(size_of::<Tick>() == size_of::<i64>(), "an `i64`, and nothing else");

impl Tick {
    /// The reading `value`, as the counter's register held it.
    #[inline]
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// The reading, as the counter's register held it.
    #[inline]
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// A signed count of counter ticks: how far apart two [`Tick`]s are.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)
)]
pub struct Ticks(pub(crate) i64);

const _: () = assert!(size_of::<Ticks>() == size_of::<i64>(), "an `i64`, and nothing else");

impl Ticks {
    /// A count of `value` ticks.
    #[inline]
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// The count of ticks.
    #[inline]
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}
