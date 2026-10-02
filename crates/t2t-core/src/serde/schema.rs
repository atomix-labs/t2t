//! The JSON schema of each spelling, inlined, since each is one string.

use alloc::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

use crate::{
    BootTime, RawUptime, TaiTimestamp, Tick, TickRate, Ticks, Timedelta, Timestamp, Uptime,
};

/// The pattern of a span's spelling: `0`, or counts with units, each at most once and coarsest
/// first, behind a lookahead for a count.
macro_rules! span_pattern {
    () => {
        "^(0|-?(?=[0-9])([0-9]+d)?([0-9]+h)?([0-9]+m)?([0-9]+s)?([0-9]+ms)?([0-9]+us)?([0-9]+ns)?)$"
    };
}

/// The pattern of RFC 3339's date and time, a fraction of one to nine digits or none, then `$zone`.
macro_rules! date_time_pattern {
    ($zone:literal) => {
        concat!(
            "^[0-9]{4}-[0-9]{2}-[0-9]{2}[Tt][0-9]{2}:[0-9]{2}:[0-9]{2}(\\.[0-9]{1,9})?",
            $zone,
            "$"
        )
    };
}

/// The pattern of a count of ticks: an integer, then ` ticks`.
macro_rules! ticks_pattern {
    () => {
        "^-?[0-9]+ ticks$"
    };
}

/// The schema of a `$type` written as its spelling: a string, described and patterned.
macro_rules! spelled_schema {
    ($type:ident, $description:literal, $pattern:expr $(, format = $format:literal)? $(,)?) => {
        impl JsonSchema for $type {
            fn schema_name() -> Cow<'static, str> {
                Cow::Borrowed(stringify!($type))
            }

            fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "string",
                    $("format": $format,)?
                    "description": $description,
                    "pattern": $pattern,
                })
            }

            fn inline_schema() -> bool {
                true
            }
        }
    };
}

spelled_schema!(
    Timestamp,
    "An RFC 3339 instant in UTC, with a fraction of one to nine digits or none, as \"2026-09-16T07:45:35Z\".",
    date_time_pattern!("[Zz]"),
    format = "date-time",
);
spelled_schema!(
    TaiTimestamp,
    "An RFC 3339 date and time in TAI, with a fraction of one to nine digits or none, as \"2026-09-16T07:46:12 TAI\".",
    date_time_pattern!(" TAI"),
);
spelled_schema!(
    Uptime,
    "The span since the monotonic clock's origin, as \"1m30s\".",
    span_pattern!(),
);
spelled_schema!(
    RawUptime,
    "The span since the raw monotonic clock's origin, as \"1m30s\".",
    span_pattern!(),
);
spelled_schema!(BootTime, "The span since boot, as \"1m30s\".", span_pattern!());
spelled_schema!(Tick, "A counter's reading, a count of ticks, as \"24 ticks\".", ticks_pattern!());
spelled_schema!(
    Timedelta,
    "A signed span: \"0\", or counts with units, each at most once and coarsest first, of d, h, m, s, ms, us, ns, as \"1m30s\".",
    span_pattern!(),
);
spelled_schema!(Ticks, "A signed count of ticks, as \"24 ticks\".", ticks_pattern!());
spelled_schema!(
    TickRate,
    "A counter's rate, a count of hertz above zero, as \"24000000 Hz\".",
    "^[0-9]*[1-9][0-9]* Hz$",
);

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use fancy_regex::Regex;
    use rstest::rstest;
    use schemars::{JsonSchema, schema_for};

    use crate::{
        BootTime, RawUptime, TaiTimestamp, Tick, TickRate, Ticks, Timedelta, Timestamp, Uptime,
    };

    /// The pattern of `T`'s schema.
    fn pattern<T: JsonSchema>() -> Regex {
        let schema = schema_for!(T);
        let pattern = schema.get("pattern").and_then(|pattern| pattern.as_str());
        // fancy-regex reads the lookahead a span's pattern holds, which the `regex` crate refuses.
        Regex::new(pattern.expect("a pattern")).expect("a pattern fancy-regex reads")
    }

    /// Asserts that `T`'s schema takes `text` as `T`'s parser does.
    fn assert_agree<T: JsonSchema + FromStr>(text: &str) {
        let takes = pattern::<T>().is_match(text).expect("a match that ends");
        assert_eq!(
            takes,
            text.parse::<T>().is_ok(),
            "the pattern takes {text:?} as the parser does"
        );
    }

    // A count past the range is the parser's alone to refuse.
    #[rstest]
    #[case::zero("0")]
    #[case::zero_seconds("0s")]
    #[case::every_unit("1d2h3m4s5ms6us7ns")]
    #[case::a_gap_inside("1m0s123ms")]
    #[case::backwards("-3us")]
    #[case::a_count_past_the_next_unit("1000ms")]
    #[case::empty("")]
    #[case::a_sign_alone("-")]
    #[case::a_backwards_zero("-0")]
    #[case::no_unit("1")]
    #[case::no_count("s")]
    #[case::a_unit_twice("1m1m")]
    #[case::finest_first("1s1m")]
    #[case::spaced("5 minutes")]
    #[case::an_unknown_unit("1msec")]
    fn the_span_pattern_takes_what_the_parser_reads(#[case] text: &str) {
        assert_agree::<Timedelta>(text);
        assert_agree::<Uptime>(text);
        assert_agree::<RawUptime>(text);
        assert_agree::<BootTime>(text);
    }

    // A date that does not exist, or one past the range, is the parser's alone to refuse.
    #[rstest]
    #[case::nanoseconds("2026-09-16T07:45:35.123456789Z")]
    #[case::either_case("1970-01-01t00:00:00z")]
    #[case::one_fraction_digit("1970-01-01T00:00:00.5Z")]
    #[case::in_tai("2026-09-16T07:46:12.123456789 TAI")]
    #[case::a_date_alone("2026-09-16")]
    #[case::no_zone("2026-09-16T07:45:35")]
    #[case::an_offset("2026-09-16T07:45:35+01:00")]
    #[case::trailing_text("2026-09-16T07:45:35Z ")]
    #[case::a_point_without_digits("2026-09-16T07:45:35.Z")]
    #[case::ten_fraction_digits("2026-09-16T07:45:35.1234567890Z")]
    #[case::a_letter_for_a_digit("2026-09-1xT07:45:35Z")]
    #[case::tai_in_lower_case("2026-09-16T07:46:12 tai")]
    fn the_instant_patterns_take_what_their_parsers_read(#[case] text: &str) {
        assert_agree::<Timestamp>(text);
        assert_agree::<TaiTimestamp>(text);
    }

    // A count past the range is the parser's alone to refuse.
    #[rstest]
    #[case::ticks("24 ticks")]
    #[case::backwards("-24 ticks")]
    #[case::hertz("24000000 Hz")]
    #[case::leading_zeros("0024 Hz")]
    #[case::zero_hertz("0 Hz")]
    #[case::no_unit("24")]
    #[case::a_sign_display_never_writes("+24 ticks")]
    #[case::one_tick("1 tick")]
    #[case::kilohertz("24 kHz")]
    fn the_count_patterns_take_what_their_parsers_read(#[case] text: &str) {
        assert_agree::<Tick>(text);
        assert_agree::<Ticks>(text);
        assert_agree::<TickRate>(text);
    }
}
