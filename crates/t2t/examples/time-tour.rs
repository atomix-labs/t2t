//! The values in one pass: instants and spans, the calendar, a stamped value, and a counter's rate.
//!
//! ```text
//! cargo run -p t2t --example time-tour
//! ```

#![expect(clippy::print_stdout, reason = "a tour prints what it finds")]

use t2t::{Tick, TickRate, Ticks, Timed, Timedelta, Timestamp};

/// The best prices on a book, worth stamping.
#[derive(Debug, Clone, Copy)]
struct Quote {
    /// The best bid, in the instrument's price ticks.
    bid: u64,
    /// The best offer, in the same.
    ask: u64,
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
    let date = captured.to_utc();
    println!(
        "that is            {}-{:02}-{:02}, {} ns past the second",
        date.year, date.month, date.day, date.nanosecond
    );
    let minute = captured.floor(Timedelta::MINUTE);
    println!("its minute         {minute:.0} to {:.0}", minute + Timedelta::MINUTE);

    // A value carries the stamp its writer captured it with, and says how old it is.
    let quote = Timed::new(captured, Quote { bid: 100, ask: 101 });
    let age = quote.elapsed(captured + Timedelta::from_millis(1_500));
    println!("quote              {} / {}, {age} old, stale: {}", quote.bid, quote.ask, age > stale);

    // A counter's ticks mean nothing until a rate says what one is worth.
    let start = Tick::new(1_000_000);
    let ticks = start + Ticks::new(4_250) - start;
    println!(
        "counter ran        {} ticks: {} at {}",
        ticks.get(),
        TickRate::GIGAHERTZ.timedelta(ticks),
        TickRate::GIGAHERTZ
    );
}
