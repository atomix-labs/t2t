//! A spelling built in a fixed array, then written to the formatter's width.

use core::fmt::{self, Alignment, Write as _};
use core::str;

/// ASCII appended into a fixed array, which each spelling sizes to its longest form.
pub(crate) struct Text<const CAPACITY: usize> {
    /// The bytes appended, then zeros.
    bytes: [u8; CAPACITY],
    /// How many bytes are appended.
    length: usize,
}

impl<const CAPACITY: usize> Text<CAPACITY> {
    /// No text yet.
    pub(crate) const fn new() -> Self {
        Self { bytes: [0; CAPACITY], length: 0 }
    }

    /// Appends `bytes`, ASCII.
    ///
    /// # Errors
    /// [`fmt::Error`], past `CAPACITY`.
    pub(crate) fn push(&mut self, bytes: &[u8]) -> fmt::Result {
        let end = self.length.checked_add(bytes.len()).ok_or(fmt::Error)?;
        self.bytes.get_mut(self.length..end).ok_or(fmt::Error)?.copy_from_slice(bytes);
        self.length = end;
        Ok(())
    }
}

/// Padded to the formatter's width with its fill and alignment, left by default.
///
/// [`Formatter::pad`](fmt::Formatter::pad) would read the precision as a truncation, where a
/// `Timestamp` reads it as its fraction's digits.
impl<const CAPACITY: usize> fmt::Display for Text<CAPACITY> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Validated whole, zeros and all: an instant then writes in 29 ns against the cut's 35 ns,
        // and a span in 46.7 ns against 46.2 ns, on a Graviton4, as
        // `benches/results/2026-10-02T19-57Z-2a8c1db-text-validation/` shows.
        let text = str::from_utf8(&self.bytes).ok().and_then(|text| text.get(..self.length));
        let text = text.ok_or(fmt::Error)?;
        let Some(width) = formatter.width() else {
            return formatter.write_str(text);
        };
        let slack = width.saturating_sub(text.len());
        let (before, after) = match formatter.align() {
            Some(Alignment::Right) => (slack, 0),
            Some(Alignment::Center) => (slack / 2, slack.div_ceil(2)),
            Some(Alignment::Left) | None => (0, slack),
        };
        let fill = formatter.fill();
        for _ in 0..before {
            formatter.write_char(fill)?;
        }
        formatter.write_str(text)?;
        for _ in 0..after {
            formatter.write_char(fill)?;
        }
        Ok(())
    }
}
