//! The serde listings of the chapter Serialization.

#[test]
// ANCHOR: spelled
fn a_value_is_written_as_its_spelling_where_a_person_reads_the_format() {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string};
    use t2t_core::{Timedelta, Timestamp};

    /// A job's settings, as a config file holds them.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Job {
        /// When the job first runs.
        start: Timestamp,
        /// How often it runs after that.
        every: Timedelta,
    }

    let job: Job = from_str(r#"{"start":"2026-09-16T07:45:00Z","every":"1m30s"}"#).expect("a job");
    assert_eq!(job.every, Timedelta::from_secs(90), "a span, from its spelling");

    let written = to_string(&job).expect("a timestamp and a span always serialize");
    let expected = r#"{"start":"2026-09-16T07:45:00.000000000Z","every":"1m30s"}"#;
    assert_eq!(written, expected, "and written back as the spellings `Display` writes");
}
// ANCHOR_END: spelled

#[test]
// ANCHOR: compact
fn a_format_no_person_reads_takes_the_count() {
    use serde_test::{Configure as _, Token, assert_tokens};
    use t2t_core::{TickRate, Timedelta, Timestamp};

    let span = Timedelta::from_secs(90);
    assert_tokens(&span.readable(), &[Token::Str("1m30s")]);
    assert_tokens(&span.compact(), &[Token::I64(90_000_000_000)]);

    assert_tokens(&Timestamp::UNIX_EPOCH.compact(), &[Token::I64(0)]);
    let rate = TickRate::from_hertz(24_000_000).expect("a rate above zero");
    assert_tokens(&rate.compact(), &[Token::U64(24_000_000)]);
}
// ANCHOR_END: compact

#[test]
// ANCHOR: units
fn a_count_in_a_named_unit_takes_a_module() {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string};
    use t2t_core::serde::{timedelta, timestamp};
    use t2t_core::{Timedelta, Timestamp};

    /// An order, as a venue's feed sends it.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Order {
        /// When the venue took it, in milliseconds since the epoch.
        #[serde(with = "timestamp::millis")]
        accepted: Timestamp,
        /// When the venue matched it, in nanoseconds since the epoch.
        #[serde(with = "timestamp::nanos")]
        matched: Timestamp,
        /// How long it rests, in seconds, where it lapses at all.
        #[serde(with = "timedelta::secs::option")]
        lifetime: Option<Timedelta>,
    }

    let feed = r#"{"accepted":1789544735123,"matched":"1789544735123456789","lifetime":"30"}"#;
    let order: Order = from_str(feed).expect("an order");
    assert_eq!(order.accepted, Timestamp::from_millis(1_789_544_735_123), "a number");
    assert_eq!(order.lifetime, Some(Timedelta::from_secs(30)), "or a decimal string");

    let written = to_string(&order).expect("each field serializes as a number or a string");
    let expected = r#"{"accepted":1789544735123,"matched":"1789544735123456789","lifetime":30}"#;
    assert_eq!(written, expected, "nanoseconds since the epoch in a string, the rest as numbers");
}
// ANCHOR_END: units

#[test]
// ANCHOR: range
fn a_count_past_the_range_is_refused_and_a_finer_one_rounded_down() {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string};
    use t2t_core::Timestamp;
    use t2t_core::serde::timestamp;

    /// A stamp, in whole seconds since the epoch.
    #[derive(Debug, Serialize, Deserialize)]
    struct Stamp {
        /// When the stamped thing was published.
        #[serde(with = "timestamp::secs")]
        published: Timestamp,
    }

    let refused = from_str::<Stamp>(r#"{"published":253402300799}"#).expect_err("the year 9999");
    let reason = refused.to_string();
    assert!(reason.starts_with("out of range error"), "past 2262: {reason}");

    let stamp = Stamp { published: Timestamp::from_millis(1_789_544_735_999) };
    let written = to_string(&stamp).expect("a count of seconds always serializes");
    assert_eq!(written, r#"{"published":1789544735}"#, "the second the moment falls in");
}
// ANCHOR_END: range
