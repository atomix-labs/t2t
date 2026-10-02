//! Spans of two kinds never mix: a counter's reading moves by its ticks, not by nanoseconds.

use t2t_core::{Tick, Timedelta};

fn main() {
    let _later = Tick::new(10) + Timedelta::SECOND;
}
