//! Each clock side by side: what it reads, and what a reading costs on this machine.
//!
//! ```text
//! cargo run --release -p t2t --features std --example clock-tour
//! ```

#![expect(clippy::print_stdout, reason = "a tour prints what it finds")]

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
    println!("Counter's rate        {}\n", counter.rate());

    println!("SystemClock           {}", SystemClock.now());
    println!("CoarseSystemClock     {}", CoarseSystemClock.now());
    #[cfg(target_os = "linux")]
    println!("TaiClock              {}", TaiClock.now());
    println!("MonotonicClock        {}", MonotonicClock.now());
    println!("CoarseMonotonicClock  {}", CoarseMonotonicClock.now());
    println!("RawMonotonicClock     {}", RawMonotonicClock.now());
    println!("BootClock             {}", BootClock.now());
    println!("ProcessCpuClock       {}", ProcessCpuClock.now());
    println!("ThreadCpuClock        {}", ThreadCpuClock.now());
    println!("Counter               {:?}\n", counter.now());

    for (name, cost) in [
        ("SystemClock", cost(&counter, &SystemClock)),
        ("CoarseSystemClock", cost(&counter, &CoarseSystemClock)),
        ("MonotonicClock", cost(&counter, &MonotonicClock)),
        ("CoarseMonotonicClock", cost(&counter, &CoarseMonotonicClock)),
        ("RawMonotonicClock", cost(&counter, &RawMonotonicClock)),
        ("BootClock", cost(&counter, &BootClock)),
        ("ProcessCpuClock", cost(&counter, &ProcessCpuClock)),
        ("ThreadCpuClock", cost(&counter, &ThreadCpuClock)),
        ("Counter", cost(&counter, &counter)),
    ] {
        println!("{name:<21} {cost} a reading");
    }
    ExitCode::SUCCESS
}

/// What one reading of `clock` costs, timed on `counter` over [`READINGS`] readings.
fn cost<C: Clock>(counter: &Counter, clock: &C) -> Timedelta {
    let start = counter.now();
    for _ in 0..READINGS {
        black_box(clock.now());
    }
    let total = counter.timedelta(counter.now() - start);
    total.checked_div(READINGS).unwrap_or(Timedelta::ZERO)
}
