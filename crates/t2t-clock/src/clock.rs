//! The one verb every clock answers.

/// A clock: what time it is now.
///
/// The type a clock reads says which timeline its readings are on, so readings of two timelines
/// never mix.
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
///     type Instant = Tick;
///
///     fn now(&self) -> Tick {
///         self.0.set(self.0.get() + 1);
///         Tick::new(self.0.get())
///     }
/// }
///
/// let clock = StepClock(Cell::new(0));
/// assert!(clock.now() < clock.now(), "each reading later than the last");
/// ```
pub trait Clock {
    /// What the clock reads.
    type Instant: Copy;

    /// The time now.
    fn now(&self) -> Self::Instant;
}

impl<C: Clock + ?Sized> Clock for &C {
    type Instant = C::Instant;

    #[inline]
    fn now(&self) -> Self::Instant {
        (**self).now()
    }
}
