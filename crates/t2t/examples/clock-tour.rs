//! Each clock side by side: what it reads, and what a reading costs on this machine.
//!
//! It runs where the OS clocks and a counter are: 64-bit Linux or macOS, on Arm or x86.
//!
//! ```text
//! cargo run --release -p t2t --features std --example clock-tour
//! ```

#![expect(clippy::print_stdout, reason = "a tour prints what it finds")]

use core::fmt::Display;
use core::hint::black_box;
use std::process::ExitCode;

use t2t::Timedelta;
#[cfg(target_os = "linux")]
use t2t::clock::TaiClock;
use t2t::clock::{
    BootClock, Clock, CoarseMonotonicClock, CoarseSystemClock, Counter, MonotonicClock,
    ProcessCpuClock, RawMonotonicClock, SystemClock, ThreadCpuClock,
};

/// The readings taken back to back to price one.
const READINGS: i64 = 100_000;

fn main() -> ExitCode {
    let counter = match Counter::discover() {
        Ok(counter) => counter,
        Err(error) => {
            #[expect(clippy::print_stderr, reason = "the person running the tour reads why")]
            {
                eprintln!("clock-tour: {error}");
            }
            return ExitCode::FAILURE;
        },
    };
    println!("{:<21} {}\n", "Counter's rate", counter.rate());
    report(&counter, "SystemClock", &SystemClock);
    report(&counter, "CoarseSystemClock", &CoarseSystemClock);
    #[cfg(target_os = "linux")]
    report(&counter, "TaiClock", &TaiClock);
    report(&counter, "MonotonicClock", &MonotonicClock);
    report(&counter, "CoarseMonotonicClock", &CoarseMonotonicClock);
    report(&counter, "RawMonotonicClock", &RawMonotonicClock);
    report(&counter, "BootClock", &BootClock);
    report(&counter, "ProcessCpuClock", &ProcessCpuClock);
    report(&counter, "ThreadCpuClock", &ThreadCpuClock);
    report(&counter, "Counter", &counter);
    ExitCode::SUCCESS
}

/// Prints what `clock` reads now, and what one reading of it costs, timed on `counter`.
fn report<C: Clock<Reading: Display>>(counter: &Counter, name: &str, clock: &C) {
    println!("{name:<21} {:<36} {} a reading", clock.now(), cost(counter, clock));
}

/// What one reading of `clock` costs, timed on `counter` over [`READINGS`] readings.
fn cost<C: Clock>(counter: &Counter, clock: &C) -> Timedelta {
    let start = counter.now();
    for _ in 0..READINGS {
        black_box(clock.now());
    }
    let total = (counter.now() - start).to_timedelta(counter.rate());
    total.checked_div(READINGS).unwrap_or(Timedelta::ZERO)
}
