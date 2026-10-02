//! Points of two timelines never mix: a wall-clock instant minus an uptime does not compile.

use t2t_core::{Timestamp, Uptime};

fn main() {
    let _span = Timestamp::from_secs(10) - Uptime::from_secs(10);
}
