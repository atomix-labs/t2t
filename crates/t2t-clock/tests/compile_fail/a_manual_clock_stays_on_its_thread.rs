//! A manual clock is read and moved on one thread: another thread cannot share it.

use std::thread;

use t2t_clock::{Clock, ManualClock};
use t2t_core::Timestamp;

fn main() {
    let clock = ManualClock::new(Timestamp::UNIX_EPOCH);
    thread::scope(|scope| {
        scope.spawn(|| clock.now());
    });
}
