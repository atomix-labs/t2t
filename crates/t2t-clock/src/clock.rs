//! The one verb every clock answers.

/// A clock: what time it is now.
///
/// The type a clock reads says which timeline its readings are on, so readings of two timelines
/// never mix: a point for a clock that names a moment, and a span for one that counts CPU time.
///
/// # Examples
/// ```
/// use core::cell::Cell;
///
/// use t2t_clock::Clock;
/// use t2t_core::Tick;
///
/// /// A clock that moves one tick at each reading.
/// struct StepClock(Cell<i64>);
///
/// impl Clock for StepClock {
///     type Reading = Tick;
///
///     fn now(&self) -> Tick {
///         self.0.set(self.0.get() + 1);
///         Tick::from_ticks(self.0.get())
///     }
/// }
///
/// let clock = StepClock(Cell::new(0));
/// assert!(clock.now() < clock.now(), "each reading later than the last");
/// ```
pub trait Clock {
    /// What the clock reads.
    type Reading: Copy;

    /// The time now.
    #[must_use]
    fn now(&self) -> Self::Reading;
}

impl<C: Clock + ?Sized> Clock for &C {
    type Reading = C::Reading;

    #[inline]
    fn now(&self) -> Self::Reading {
        (**self).now()
    }
}
