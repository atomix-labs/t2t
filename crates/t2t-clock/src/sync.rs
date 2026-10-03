//! The atomics the crate shares across threads: `core`'s, or loom's under `--cfg loom`, so the
//! clocks and their models are one body.

#[cfg(not(loom))]
pub(crate) use core::sync::atomic::{AtomicI64, Ordering};

#[cfg(loom)]
pub(crate) use loom::sync::atomic::{AtomicI64, Ordering};
