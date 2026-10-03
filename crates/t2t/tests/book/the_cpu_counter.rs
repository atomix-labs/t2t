//! The listings of the chapter The CPU Counter.

#[test]
#[cfg_attr(miri, ignore = "Miri cannot execute the counter's read")]
// ANCHOR: discover
fn the_counter_is_discovered_or_refused_with_its_reason() {
    use t2t::Timedelta;
    use t2t::clock::{Clock, Counter};

    match Counter::discover() {
        Ok(counter) => {
            let start = counter.now();
            let ticks = counter.now() - start;
            let took = ticks.to_timedelta(counter.rate());
            println!("two reads {ticks} apart at {}: {took}", counter.rate());
            assert!(took < Timedelta::from_millis(1), "two reads back to back");
        },
        // A virtual machine may promise no invariant counter, and a CPU may report no rate.
        Err(refusal) => println!("no counter to time with: {refusal}"),
    }
}
// ANCHOR_END: discover

#[test]
#[cfg_attr(miri, ignore = "Miri cannot execute the counter's read")]
// ANCHOR: known-rate
fn a_counter_at_a_known_rate_needs_no_discovery() {
    use t2t::TickRate;
    use t2t::clock::{Clock, Counter};

    // A rate taken from elsewhere: a config, or a discovery another process made.
    let rate: TickRate = "1000000000 Hz".parse().expect("a rate above zero");
    let counter = Counter::new(rate);

    let start = counter.now();
    assert!(counter.now() >= start, "the counter never runs backwards");
    assert_eq!(counter.rate(), TickRate::GIGAHERTZ, "and runs at the rate it was given");
}
// ANCHOR_END: known-rate

#[test]
// ANCHOR: rate
fn a_rate_turns_ticks_into_a_span_and_back() {
    use t2t::{TickRate, Tickdelta, Tickstamp, Timedelta};

    // Apple silicon's counter runs at 24 MHz.
    let rate = TickRate::from_hertz(24_000_000).expect("a rate above zero");
    let (start, end) = (Tickstamp::from_ticks(1_000_000), Tickstamp::from_ticks(1_000_024));

    let ticks = end - start;
    assert_eq!(ticks, Tickdelta::from_ticks(24), "a reading minus a reading is a span of ticks");
    assert_eq!(ticks.to_timedelta(rate), Timedelta::MICROSECOND, "worth a microsecond at 24 MHz");
    assert_eq!(Timedelta::SECOND.to_tickdelta(rate), Tickdelta::from_ticks(24_000_000), "and back");
    assert_eq!(TickRate::from_hertz(0), None, "a counter that never ticks has no rate");
}
// ANCHOR_END: rate

#[test]
// ANCHOR: accuracy
fn a_conversion_is_within_one_unit_of_the_exact_quotient() {
    use t2t::{TickRate, Tickdelta, Timedelta};

    // 3 GHz: a nanosecond is three ticks, and a tick a third of a nanosecond.
    let rate = TickRate::from_hertz(3_000_000_000).expect("a rate above zero");
    let exact = Tickdelta::from_ticks(3_000).to_timedelta(rate);
    assert_eq!(exact, Timedelta::MICROSECOND, "a multiple of the ratio converts exactly");

    // 1,000 ticks are 333.33 ns: the result is 333 or 334, never further off.
    let nanos = Tickdelta::from_ticks(1_000).to_timedelta(rate).as_nanos();
    assert!((333..=334).contains(&nanos), "within a nanosecond of 333.33: {nanos}");

    // A count whose span is past what nanoseconds hold saturates, as every operator does.
    let slow = TickRate::from_hertz(1).expect("a rate above zero");
    assert_eq!(Tickdelta::MAX.to_timedelta(slow), Timedelta::MAX, "a tick a second, for ever");
}
// ANCHOR_END: accuracy
