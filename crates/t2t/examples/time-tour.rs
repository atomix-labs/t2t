//! The values in one pass: instants and spans, the calendar, a stamped value, and a counter's rate.
//!
//! ```text
//! cargo run -p t2t --example time-tour
//! ```

#![expect(clippy::print_stdout, reason = "a tour prints what it finds")]

use t2t::{TickRate, Tickdelta, Tickstamp, Timed, Timedelta, Timestamp};

/// What a weather station measured, worth stamping.
#[derive(Debug, Clone, Copy)]
struct Weather {
    /// The temperature, in degrees Celsius.
    celsius: i16,
    /// The relative humidity, in percent.
    humidity: u8,
}

fn main() {
    // A span is written and read the way a config spells it.
    let stale = Timedelta::SECOND;
    println!("stale after        {stale}");
    println!("\"1h30m\" reads as   {:?}", "1h30m".parse::<Timedelta>());

    // An instant is a point; the difference of two is a span.
    let published = Timestamp::from_secs(1_700_000_000);
    let captured = published + Timedelta::from_micros(850);
    println!("published          {published}");
    println!("captured           {captured:.6}");
    println!("on the wire        {}", captured - published);

    // The same instant, read as a date; and the minute it falls in.
    let date_time = captured.to_utc();
    println!(
        "that is            {}-{:02}-{:02}, {} ns past the second",
        date_time.year, date_time.month, date_time.day, date_time.nanosecond
    );
    let minute = captured.floor(Timedelta::MINUTE);
    println!("its minute         {minute:.0} to {:.0}", minute + Timedelta::MINUTE);

    // A value carries the stamp its writer captured it with, and says how old it is.
    let sample = Timed::new(captured, Weather { celsius: 21, humidity: 40 });
    let age = sample.elapsed(captured + Timedelta::from_millis(1_500));
    println!(
        "sample             {} °C, {}%, {age} old, stale: {}",
        sample.celsius,
        sample.humidity,
        age > stale
    );

    // A counter's ticks mean nothing until a rate says what one is worth.
    let start = Tickstamp::from_ticks(1_000_000);
    let ticks = start + Tickdelta::from_ticks(4_250) - start;
    let rate = TickRate::GIGAHERTZ;
    println!("counter ran        {ticks}: {} at {rate}", ticks.to_timedelta(rate));
}
