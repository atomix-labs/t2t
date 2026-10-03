//! What converting a time costs: a rate, a calendar reading, and a spelling, each both ways, timed
//! over 1024 varied values, so the figure is a conversion's own cost and not one input's.
//!
//! ```text
//! cargo bench -p t2t-core --bench convert
//! ```

#![expect(clippy::expect_used, reason = "a bench has no caller to hand a refusal to")]

use core::fmt::Write as _;
use core::hint::black_box;

use divan::counter::ItemsCount;
use divan::{Bencher, bench, main as run_benches};
use proptest::prelude::RngCore as _;
use proptest::test_runner::{RngAlgorithm, TestRng};
use t2t_core::{TickRate, Tickdelta, Timedelta, Timestamp, UtcDateTime};

/// The values each bench converts.
const COUNT: usize = 1024;

/// Apple silicon's counter rate, whose ratio to a nanosecond is no power of two.
const RATE: TickRate = TickRate::from_hertz(24_000_000).expect("a nonzero rate");

fn main() {
    run_benches();
}

/// [`COUNT`] counts spread over every `i64`, the same on every run.
fn counts() -> Vec<i64> {
    let mut random = TestRng::deterministic_rng(RngAlgorithm::ChaCha);
    (0..COUNT).map(|_| random.next_u64().cast_signed()).collect()
}

/// [`COUNT`] spans of up to about 19 hours either way, the spread a config or a log holds.
fn spans() -> Vec<Timedelta> {
    counts().into_iter().map(|count| Timedelta::from_nanos(count >> 17)).collect()
}

/// [`COUNT`] instants over every year a timestamp holds.
fn instants() -> Vec<Timestamp> {
    counts().into_iter().map(Timestamp::from_nanos).collect()
}

/// Ticks to nanoseconds.
#[bench]
fn ticks_to_timedelta(bencher: Bencher<'_, '_>) {
    let ticks: Vec<Tickdelta> = counts().into_iter().map(Tickdelta::from_ticks).collect();
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        black_box(&ticks)
            .iter()
            .map(|&count| count.to_timedelta(RATE).as_nanos())
            .fold(0, i64::wrapping_add)
    });
}

/// Nanoseconds to ticks.
#[bench]
fn timedelta_to_ticks(bencher: Bencher<'_, '_>) {
    let spans = spans();
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        black_box(&spans)
            .iter()
            .map(|&span| span.to_tickdelta(RATE).as_ticks())
            .fold(0, i64::wrapping_add)
    });
}

/// An instant read as a date and a time of day.
#[bench]
fn timestamp_to_utc(bencher: Bencher<'_, '_>) {
    let instants = instants();
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        black_box(&instants)
            .iter()
            .map(|instant| instant.to_utc().nanosecond)
            .fold(0, u32::wrapping_add)
    });
}

/// A date and a time of day read back as an instant.
#[bench]
fn utc_to_timestamp(bencher: Bencher<'_, '_>) {
    let date_times: Vec<UtcDateTime> = instants().into_iter().map(Timestamp::to_utc).collect();
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        black_box(&date_times)
            .iter()
            .filter_map(|date_time| date_time.to_timestamp())
            .map(Timestamp::as_nanos)
            .fold(0, i64::wrapping_add)
    });
}

/// An instant written as RFC 3339, into a string kept between iterations.
#[bench]
fn write_timestamp(bencher: Bencher<'_, '_>) {
    let instants = instants();
    let mut text = String::with_capacity(40);
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        for instant in black_box(&instants) {
            text.clear();
            write!(text, "{instant}").expect("a string takes any write");
        }
    });
}

/// RFC 3339 read as an instant.
#[bench]
fn parse_timestamp(bencher: Bencher<'_, '_>) {
    let texts: Vec<String> = instants().iter().map(ToString::to_string).collect();
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        black_box(&texts)
            .iter()
            .filter_map(|text| text.parse().ok())
            .map(Timestamp::as_nanos)
            .fold(0, i64::wrapping_add)
    });
}

/// A span written in its spelling, into a string kept between iterations.
#[bench]
fn write_timedelta(bencher: Bencher<'_, '_>) {
    let spans = spans();
    let mut text = String::with_capacity(40);
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        for span in black_box(&spans) {
            text.clear();
            write!(text, "{span}").expect("a string takes any write");
        }
    });
}

/// A span read from its spelling.
#[bench]
fn parse_timedelta(bencher: Bencher<'_, '_>) {
    let texts: Vec<String> = spans().iter().map(ToString::to_string).collect();
    bencher.counter(ItemsCount::new(COUNT)).bench_local(|| {
        black_box(&texts)
            .iter()
            .filter_map(|text| text.parse().ok())
            .map(Timedelta::as_nanos)
            .fold(0, i64::wrapping_add)
    });
}
