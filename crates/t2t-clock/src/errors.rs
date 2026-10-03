//! Why the CPU's counter cannot be read as time.

use derive_more::{Display, Error};

/// Why [`Counter::discover`](crate::Counter::discover) found no counter to read as time.
///
/// # Examples
/// ```
/// use t2t_clock::CounterError;
///
/// let refusal = CounterError::ImplausibleRate { hertz: 42 };
/// assert_eq!(
///     refusal.to_string(),
///     "counter error: a rate of 42 Hz is outside 1 MHz to 10 GHz",
///     "it names the rate it refused"
/// );
/// ```
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CounterError {
    /// The time-stamp counter's rate follows the core's, so its ticks have no fixed length.
    #[display("counter error: the time-stamp counter's rate follows the core's")]
    NotInvariant,
    /// The CPU reports no rate, and none could be measured.
    #[display("counter error: the CPU reports no rate, and none could be measured")]
    NoRate,
    /// The rate is outside what any counter runs at.
    #[display("counter error: a rate of {hertz} Hz is outside 1 MHz to 10 GHz")]
    ImplausibleRate {
        /// The rate the CPU reported, or that was measured, in ticks a second.
        hertz: u64,
    },
}

#[cfg(test)]
mod tests {
    use alloc::string::{String, ToString as _};

    use rstest::rstest;

    use crate::CounterError;

    #[rstest]
    #[case::not_invariant(
        CounterError::NotInvariant.to_string(),
        "counter error: the time-stamp counter's rate follows the core's"
    )]
    #[case::no_rate(
        CounterError::NoRate.to_string(),
        "counter error: the CPU reports no rate, and none could be measured"
    )]
    #[case::implausible_rate(
        CounterError::ImplausibleRate { hertz: 42 }.to_string(),
        "counter error: a rate of 42 Hz is outside 1 MHz to 10 GHz"
    )]
    fn each_refusal_says_why(#[case] message: String, #[case] text: &str) {
        assert_eq!(message, text, "the message names what went wrong");
    }
}
