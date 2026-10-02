//! Conversions to and from std's and other time crates' values, each crate behind its own
//! feature.

#[cfg(feature = "chrono-04")]
mod chrono;
#[cfg(feature = "jiff-02")]
mod jiff;
#[cfg(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
))]
mod system_time;
#[cfg(feature = "time-03")]
mod time;
