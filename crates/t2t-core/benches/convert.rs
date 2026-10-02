//! What converting a time costs: a rate, a calendar reading, and a spelling, each both ways.
//!
//! ```text
//! cargo bench -p t2t-core --bench convert
//! ```

#![expect(clippy::missing_const_for_fn, reason = "divan times each function at run time")]

use core::fmt::Write as _;
use core::hint::black_box;

use divan::Bencher;
use t2t_core::{
    ParseTimedeltaError, ParseTimestampError, TickRate, Ticks, Timedelta, Timestamp, UtcDateTime,
};

/// Apple silicon's counter rate, whose ratio to a nanosecond is no power of two.
const RATE: TickRate = TickRate::new(24_000_000).unwrap();

/// An instant with every digit of its fraction set.
const INSTANT: Timestamp = Timestamp::from_nanos(1_789_544_735_123_456_789);

/// A span with a count in every unit below the minute.
const SPAN: Timedelta = Timedelta::from_nanos(67_123_456_789);

fn main() {
    divan::main();
}

/// Ticks to nanoseconds.
#[divan::bench]
fn ticks_to_timedelta() -> Timedelta {
    RATE.timedelta(black_box(Ticks::new(123_456_789)))
}

/// Nanoseconds to ticks.
#[divan::bench]
fn timedelta_to_ticks() -> Ticks {
    RATE.ticks(black_box(Timedelta::from_nanos(123_456_789)))
}

/// An instant read as a date and a time of day.
#[divan::bench]
fn timestamp_to_utc() -> UtcDateTime {
    black_box(INSTANT).to_utc()
}

/// A date and a time of day read back as an instant.
#[divan::bench]
fn utc_to_timestamp() -> Option<Timestamp> {
    black_box(INSTANT.to_utc()).to_timestamp()
}

/// An instant written as RFC 3339, into a string kept between iterations.
#[divan::bench]
fn write_rfc3339(bencher: Bencher<'_, '_>) {
    let mut text = String::with_capacity(32);
    bencher.bench_local(|| {
        text.clear();
        write!(text, "{}", black_box(INSTANT))
    });
}

/// RFC 3339 read as an instant.
#[divan::bench]
fn parse_rfc3339() -> Result<Timestamp, ParseTimestampError> {
    black_box("2026-09-16T07:45:35.123456789Z").parse()
}

/// A span written in its spelling, into a string kept between iterations.
#[divan::bench]
fn write_timedelta(bencher: Bencher<'_, '_>) {
    let mut text = String::with_capacity(32);
    bencher.bench_local(|| {
        text.clear();
        write!(text, "{}", black_box(SPAN))
    });
}

/// A span read from its spelling.
#[divan::bench]
fn parse_timedelta() -> Result<Timedelta, ParseTimedeltaError> {
    black_box("1m7s123ms456us789ns").parse()
}
