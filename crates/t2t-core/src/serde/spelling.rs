//! Each value on the wire: the string its `Display` writes, where a person reads the format, and
//! its count, where none does.

use core::fmt;
use core::str::FromStr;

use serde_core::de::{self, Unexpected, Visitor};
use serde_core::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    BootUptime, RawUptime, TaiTimestamp, TickRate, Tickdelta, Tickstamp, Timedelta, Timestamp,
    Uptime,
};

/// Serializes `$type` as the string its `Display` and `FromStr` agree on, where a person reads the
/// format, and as `$as_count`'s `$count` where none does, read back through `$from_count`.
macro_rules! spelled {
    (
        $type:ident,
        $visitor:ident,
        $expecting:literal,
        $count:ty,
        $as_count:path,
        $from_count:expr $(,)?
    ) => {
        /// Reads the value from its spelling.
        struct $visitor;

        impl Visitor<'_> for $visitor {
            type Value = $type;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str($expecting)
            }

            fn visit_str<E: de::Error>(self, text: &str) -> Result<$type, E> {
                $type::from_str(text).map_err(E::custom)
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                if deserializer.is_human_readable() {
                    deserializer.deserialize_str($visitor)
                } else {
                    ($from_count)(<$count>::deserialize(deserializer)?)
                }
            }
        }

        impl Serialize for $type {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                if serializer.is_human_readable() {
                    serializer.collect_str(self)
                } else {
                    $as_count(*self).serialize(serializer)
                }
            }
        }
    };
}

spelled!(
    Timestamp,
    TimestampVisitor,
    "an RFC 3339 instant in UTC, as \"2026-09-16T07:45:35Z\"",
    i64,
    Timestamp::as_nanos,
    |nanos| Ok(Timestamp::from_nanos(nanos)),
);
spelled!(
    TaiTimestamp,
    TaiTimestampVisitor,
    "an RFC 3339 date and time in TAI, as \"2026-09-16T07:46:12 TAI\"",
    i64,
    TaiTimestamp::as_nanos,
    |nanos| Ok(TaiTimestamp::from_nanos(nanos)),
);
spelled!(
    Uptime,
    UptimeVisitor,
    "the span since the origin, as \"1m30s\"",
    i64,
    Uptime::as_nanos,
    |nanos| Ok(Uptime::from_nanos(nanos)),
);
spelled!(
    RawUptime,
    RawUptimeVisitor,
    "the span since the origin, as \"1m30s\"",
    i64,
    RawUptime::as_nanos,
    |nanos| Ok(RawUptime::from_nanos(nanos)),
);
spelled!(
    BootUptime,
    BootUptimeVisitor,
    "the span since boot, as \"1m30s\"",
    i64,
    BootUptime::as_nanos,
    |nanos| Ok(BootUptime::from_nanos(nanos)),
);
spelled!(
    Tickstamp,
    TickstampVisitor,
    "a count of ticks, as \"24 ticks\"",
    i64,
    Tickstamp::as_ticks,
    |ticks| Ok(Tickstamp::from_ticks(ticks)),
);
spelled!(
    Timedelta,
    TimedeltaVisitor,
    "a span, as \"1m30s\"",
    i64,
    Timedelta::as_nanos,
    |nanos| Ok(Timedelta::from_nanos(nanos)),
);
spelled!(
    Tickdelta,
    TickdeltaVisitor,
    "a count of ticks, as \"24 ticks\"",
    i64,
    Tickdelta::as_ticks,
    |ticks| { Ok(Tickdelta::from_ticks(ticks)) },
);
// A zero count is refused as serde refuses one for a `NonZeroU64`.
spelled!(
    TickRate,
    TickRateVisitor,
    "a count of hertz above zero, as \"24000000 Hz\"",
    u64,
    TickRate::as_hertz,
    |hertz| {
        TickRate::from_hertz(hertz).ok_or_else(|| {
            de::Error::invalid_value(Unexpected::Unsigned(hertz), &"a count of hertz above zero")
        })
    },
);

#[cfg(test)]
mod tests {
    use serde_test::{
        Compact, Configure as _, Readable, Token, assert_de_tokens_error, assert_tokens,
    };

    use crate::{
        BootUptime, RawUptime, TaiTimestamp, TickRate, Tickdelta, Tickstamp, Timedelta, Timestamp,
        Uptime,
    };

    #[test]
    fn each_value_round_trips_as_its_spelling() {
        let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
        assert_tokens(&instant.readable(), &[Token::Str("2026-09-16T07:45:35.123456789Z")]);
        let tai = TaiTimestamp::from_secs(1_789_544_772);
        assert_tokens(&tai.readable(), &[Token::Str("2026-09-16T07:46:12.000000000 TAI")]);
        assert_tokens(&Uptime::from_secs(90).readable(), &[Token::Str("1m30s")]);
        assert_tokens(&RawUptime::from_secs(90).readable(), &[Token::Str("1m30s")]);
        assert_tokens(&BootUptime::from_secs(90).readable(), &[Token::Str("1m30s")]);
        assert_tokens(&Tickstamp::from_ticks(24).readable(), &[Token::Str("24 ticks")]);
        assert_tokens(&Timedelta::from_mins(5).readable(), &[Token::Str("5m")]);
        assert_tokens(&Tickdelta::from_ticks(-24).readable(), &[Token::Str("-24 ticks")]);
        let rate = TickRate::from_hertz(24_000_000).expect("a nonzero rate");
        assert_tokens(&rate.readable(), &[Token::Str("24000000 Hz")]);
    }

    #[test]
    fn a_format_no_person_reads_takes_each_value_as_its_count() {
        let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
        assert_tokens(&instant.compact(), &[Token::I64(1_789_544_735_123_456_789)]);
        assert_tokens(&TaiTimestamp::from_nanos(7).compact(), &[Token::I64(7)]);
        assert_tokens(&Uptime::from_nanos(7).compact(), &[Token::I64(7)]);
        assert_tokens(&RawUptime::from_nanos(7).compact(), &[Token::I64(7)]);
        assert_tokens(&BootUptime::from_nanos(7).compact(), &[Token::I64(7)]);
        assert_tokens(&Tickstamp::from_ticks(7).compact(), &[Token::I64(7)]);
        assert_tokens(&Timedelta::from_nanos(-7).compact(), &[Token::I64(-7)]);
        assert_tokens(&Tickdelta::from_ticks(-7).compact(), &[Token::I64(-7)]);
        let rate = TickRate::from_hertz(24_000_000).expect("a nonzero rate");
        assert_tokens(&rate.compact(), &[Token::U64(24_000_000)]);
    }

    #[test]
    fn a_spelling_that_does_not_read_is_refused_with_its_reason() {
        assert_de_tokens_error::<Readable<Timedelta>>(
            &[Token::Str("5 minutes")],
            "parse timedelta error: expected `0`, or counts with units from coarsest to finest, \
             as `1m30s`",
        );
        assert_de_tokens_error::<Compact<TickRate>>(
            &[Token::U64(0)],
            "invalid value: integer `0`, expected a count of hertz above zero",
        );
    }
}
