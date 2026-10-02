//! Why the CPU's counter cannot be read as time.

use thiserror::Error;

/// Why [`Counter::discover`](crate::Counter::discover) found no counter to read as time.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CounterError {
    /// The time-stamp counter's rate follows the core's, so its ticks have no fixed length.
    #[error("counter error: the time-stamp counter's rate follows the core's")]
    NotInvariant,
    /// The CPU reports no rate, and none could be measured.
    #[error("counter error: the CPU reports no rate, and none could be measured")]
    NoRate,
    /// The rate is outside what any counter runs at.
    #[error("counter error: a rate of {ticks_per_second} Hz is outside 1 MHz to 10 GHz")]
    ImplausibleRate {
        /// The rate the CPU reported, or that was measured.
        ticks_per_second: u64,
    },
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use crate::CounterError;

    #[test]
    fn each_refusal_says_what_was_wrong() {
        assert_eq!(
            CounterError::NotInvariant.to_string(),
            "counter error: the time-stamp counter's rate follows the core's"
        );
        assert_eq!(
            CounterError::NoRate.to_string(),
            "counter error: the CPU reports no rate, and none could be measured"
        );
        assert_eq!(
            CounterError::ImplausibleRate { ticks_per_second: 42 }.to_string(),
            "counter error: a rate of 42 Hz is outside 1 MHz to 10 GHz"
        );
    }
}
