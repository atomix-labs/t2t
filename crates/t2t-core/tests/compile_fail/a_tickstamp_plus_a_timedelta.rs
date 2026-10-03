//! Spans of two kinds never mix: a counter's reading moves by its ticks, not by nanoseconds.

use t2t_core::{Tickstamp, Timedelta};

fn main() {
    let _later = Tickstamp::from_ticks(10) + Timedelta::SECOND;
}
