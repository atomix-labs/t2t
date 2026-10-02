//! The JSON schema of each spelling, inlined, since each is one string.

use alloc::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

use crate::{Timedelta, Timestamp};

impl JsonSchema for Timedelta {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Timedelta")
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "A signed span: \"0\", or counts with units, each at most once and coarsest first, of d, h, m, s, ms, us, ns, as \"250ms\" or \"1h30m\".",
            "pattern": "^(0|-?(?=[0-9])([0-9]+d)?([0-9]+h)?([0-9]+m)?([0-9]+s)?([0-9]+ms)?([0-9]+us)?([0-9]+ns)?)$"
        })
    }

    fn inline_schema() -> bool {
        true
    }
}

impl JsonSchema for Timestamp {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Timestamp")
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "format": "date-time",
            "description": "An RFC 3339 instant in UTC, with a fraction of one to nine digits or none, as \"2026-09-16T07:45:35.123Z\".",
            "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}[Tt][0-9]{2}:[0-9]{2}:[0-9]{2}(\\.[0-9]{1,9})?[Zz]$"
        })
    }

    fn inline_schema() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use fancy_regex::Regex;
    use rstest::rstest;
    use schemars::{JsonSchema, schema_for};

    use crate::{Timedelta, Timestamp};

    /// Whether `T`'s schema pattern takes `text`; fancy-regex reads the lookahead a span's holds,
    /// which the `regex` crate refuses.
    fn pattern_takes<T: JsonSchema>(text: &str) -> bool {
        let schema = schema_for!(T);
        let pattern = schema.get("pattern").and_then(|pattern| pattern.as_str());
        let pattern = Regex::new(pattern.expect("a pattern")).expect("a pattern fancy-regex reads");
        pattern.is_match(text).expect("a match that ends")
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
        assert_eq!(
            pattern_takes::<Timedelta>(text),
            text.parse::<Timedelta>().is_ok(),
            "the pattern takes {text:?} as the parser does"
        );
    }

    // A date that does not exist, or one past the range, is the parser's alone to refuse.
    #[rstest]
    #[case::nanoseconds("2026-09-16T07:45:35.123456789Z")]
    #[case::either_case("1970-01-01t00:00:00z")]
    #[case::one_fraction_digit("1970-01-01T00:00:00.5Z")]
    #[case::a_date_alone("2026-09-16")]
    #[case::no_zone("2026-09-16T07:45:35")]
    #[case::an_offset("2026-09-16T07:45:35+01:00")]
    #[case::trailing_text("2026-09-16T07:45:35Z ")]
    #[case::a_point_without_digits("2026-09-16T07:45:35.Z")]
    #[case::ten_fraction_digits("2026-09-16T07:45:35.1234567890Z")]
    #[case::a_letter_for_a_digit("2026-09-1xT07:45:35Z")]
    fn the_instant_pattern_takes_what_the_parser_reads(#[case] text: &str) {
        assert_eq!(
            pattern_takes::<Timestamp>(text),
            text.parse::<Timestamp>().is_ok(),
            "the pattern takes {text:?} as the parser does"
        );
    }
}
