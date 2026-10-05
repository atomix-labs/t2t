//! A hardware counter's readings, and the span between two.

use core::str::FromStr;

use derive_more::{Debug, Display};
#[cfg(feature = "zerocopy-08")]
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::spelling::{Count, read_count};
use crate::{ParseTickdeltaError, TickRate, Timedelta};

/// A point on a hardware counter, in ticks since an origin the hardware chose.
///
/// The origin means nothing, so only the difference of two readings does, and only with the
/// counter's [`TickRate`]. Every core and process on a machine reads one counter, so a reading one
/// process took, another may subtract from. It is written as its count, `1000 ticks`, and parses
/// back from it.
///
/// # Examples
/// ```
/// use t2t_core::{TickRate, Tickdelta, Tickstamp, Timedelta};
///
/// let start = Tickstamp::from_ticks(1_000);
/// let end = start + Tickdelta::from_ticks(24);
///
/// assert_eq!(end - start, Tickdelta::from_ticks(24), "a reading minus a reading");
/// let rate = TickRate::from_hertz(24_000_000).expect("a nonzero rate");
/// assert_eq!((end - start).to_timedelta(rate), Timedelta::MICROSECOND, "worth a span");
/// ```
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[display("{}", Count(*_0, " ticks"))]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy-08", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct Tickstamp(pub(crate) i64);

impl Tickstamp {
    /// The point `ticks` ticks after the counter's origin, as its register holds it.
    #[inline]
    #[must_use]
    pub const fn from_ticks(ticks: i64) -> Self {
        Self(ticks)
    }

    /// Ticks since the counter's origin, as its register holds them.
    #[inline]
    #[must_use]
    pub const fn as_ticks(self) -> i64 {
        self.0
    }
}

/// A signed span of counter ticks: how far apart two [`Tickstamp`]s are.
///
/// Its operators saturate, each with a `checked_*` twin, as a [`Timedelta`]'s do; a [`TickRate`]
/// turns it into one. It is written as its count, `24 ticks`, and parses back from it.
///
/// # Examples
/// ```
/// use t2t_core::{TickRate, Tickdelta, Timedelta};
///
/// let span: Tickdelta = "3000 ticks".parse()?;
/// let rate = TickRate::from_hertz(3_000_000_000).expect("a nonzero rate");
/// assert_eq!(span.to_timedelta(rate), Timedelta::MICROSECOND, "3000 ticks at 3 GHz");
/// # Ok::<(), t2t_core::ParseTickdeltaError>(())
/// ```
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[display("{}", Count(*_0, " ticks"))]
#[debug("{self}")]
#[cfg_attr(feature = "zerocopy-08", derive(FromBytes, IntoBytes, Immutable, KnownLayout))]
pub struct Tickdelta(pub(crate) i64);

impl Tickdelta {
    /// A span of `ticks` ticks.
    #[inline]
    #[must_use]
    pub const fn from_ticks(ticks: i64) -> Self {
        Self(ticks)
    }

    /// The span in ticks.
    #[inline]
    #[must_use]
    pub const fn as_ticks(self) -> i64 {
        self.0
    }

    /// How long the span lasts at `rate`: a multiply and a shift, to within a nanosecond.
    #[inline]
    #[must_use]
    pub const fn to_timedelta(self, rate: TickRate) -> Timedelta {
        Timedelta(rate.ticks_to_nanos(self.0))
    }
}

/// Reading a count of ticks from the spelling `Display` writes.
macro_rules! read_as_ticks {
    ($count:ident) => {
        /// Reads what [`Display`](core::fmt::Display) writes: a count, then ` ticks`.
        impl FromStr for $count {
            type Err = ParseTickdeltaError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                let count = text.strip_suffix(" ticks").and_then(read_count);
                count.map(Self).ok_or(ParseTickdeltaError)
            }
        }
    };
}

read_as_ticks!(Tickstamp);
read_as_ticks!(Tickdelta);

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;

    use rstest::rstest;

    use crate::{ParseTickdeltaError, Tickdelta, Tickstamp};

    #[test]
    fn a_count_of_ticks_reads_back_from_its_spelling() {
        let span = Tickdelta::from_ticks(-24);
        assert_eq!(span.to_string(), "-24 ticks", "written as its count");
        assert_eq!("-24 ticks".parse(), Ok(span), "and read back");
        assert_eq!(
            "1000 ticks".parse(),
            Ok(Tickstamp::from_ticks(1_000)),
            "a reading the same way"
        );
    }

    #[test]
    fn a_width_pads_the_count_with_its_unit() {
        assert_eq!(format!("[{:>10}]", Tickdelta::from_ticks(24)), "[  24 ticks]", "a span");
        assert_eq!(format!("[{:<10}]", Tickstamp::from_ticks(24)), "[24 ticks  ]", "and a reading");
    }

    #[rstest]
    #[case::no_unit("24")]
    #[case::a_sign_display_never_writes("+24 ticks")]
    #[case::one_tick("1 tick")]
    #[case::no_space("24ticks")]
    #[case::past_the_range("9223372036854775808 ticks")]
    fn a_malformed_count_of_ticks_is_refused(#[case] text: &str) {
        assert_eq!(text.parse::<Tickdelta>(), Err(ParseTickdeltaError), "{text:?} is refused");
    }
}
