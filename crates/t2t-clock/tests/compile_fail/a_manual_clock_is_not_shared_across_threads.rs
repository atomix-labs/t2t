//! A manual clock may move to another thread, but two threads never share one.

use std::thread;

use t2t_clock::{Clock, ManualClock};
use t2t_core::Timestamp;

fn main() {
    let clock = ManualClock::new(Timestamp::UNIX_EPOCH);
    thread::scope(|scope| {
        scope.spawn(|| clock.now());
    });
}
