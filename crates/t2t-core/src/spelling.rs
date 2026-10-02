//! What the spellings share: the padding to a formatter's width, and a count with its unit,
//! written and read back.

use core::fmt::{self, Write as _};
use core::str::FromStr;

use arrayvec::ArrayString;
use itoa::{Buffer, Integer};
use powerfmt::ext::FormatterExt as _;

/// The longest spelling of a count with its unit, an `i64::MIN` of ticks.
const LONGEST_COUNT: usize = "-9223372036854775808 ticks".len();

/// Writes `text` to the formatter's width, with its fill and alignment, left by default, and
/// never cut to the precision, which an instant reads as its fraction's digits.
#[inline]
pub(crate) fn pad(formatter: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    // Without a width, as nearly every write is, the text goes straight to the formatter.
    if formatter.width().is_none() {
        return formatter.write_str(text);
    }
    formatter.pad_with_width(text.len(), format_args!("{text}"))
}

/// A count, then its unit, written as one spelling, so a width pads the two together: what a
/// [`Tick`](crate::Tick), a [`Ticks`](crate::Ticks) and a [`TickRate`](crate::TickRate) display.
pub(crate) struct Count<I>(pub(crate) I, pub(crate) &'static str);

impl<I: Integer> fmt::Display for Count<I> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = ArrayString::<LONGEST_COUNT>::new();
        text.write_str(Buffer::new().format(self.0))?;
        text.write_str(self.1)?;
        pad(formatter, &text)
    }
}

/// The count `text` spells as `Display` writes one: digits, after a `-` for a negative count, and
/// never a `+`.
#[inline]
pub(crate) fn read_count<T: FromStr>(text: &str) -> Option<T> {
    if text.starts_with('+') { None } else { text.parse().ok() }
}
