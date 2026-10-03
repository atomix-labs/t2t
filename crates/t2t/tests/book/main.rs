//! The book's listings, each a test, so a page shows code that compiles and runs.
//!
//! A page includes a listing by the anchor around it; a chapter's listings are in the module named
//! for it. Those that read the operating system's clocks are under `os`.

#[cfg(test)]
mod dates_and_spellings;
#[cfg(test)]
#[cfg(all(
    feature = "std",
    target_pointer_width = "64",
    any(target_os = "linux", target_os = "macos")
))]
mod os;
#[cfg(test)]
mod points_and_spans;
#[cfg(test)]
mod stamped_values;
#[cfg(test)]
#[cfg(target_has_atomic = "64")]
mod testing_with_manual_clocks;
#[cfg(test)]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
mod the_cpu_counter;
