//! Names the conditions t2t-clock's items share, so each is written once.

use cfg_aliases::cfg_aliases;

/// Declares each alias, with the `check-cfg` that lets rustc know it. `lib.rs` says each again in
/// words for docs.rs, on the re-exports it gates.
fn main() {
    cfg_aliases! {
        // The OS clocks: `clock_gettime`, with a 64-bit `timespec`, on Linux and macOS.
        os_clocks: {
            all(
                feature = "std",
                target_pointer_width = "64",
                any(target_os = "linux", target_os = "macos")
            )
        },
        // International Atomic Time, which Linux alone has a clock for.
        tai_clock: { all(os_clocks, target_os = "linux") },
        // A CPU counter one instruction reads.
        counter: { any(target_arch = "aarch64", target_arch = "x86_64") },
        // The clock shared across threads, which keeps its reading in a 64-bit atomic.
        atomic_clock: { target_has_atomic = "64" },
    }
}
