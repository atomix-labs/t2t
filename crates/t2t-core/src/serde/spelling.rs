//! A time on the wire as the string its `Display` writes.

use core::fmt;
use core::str::FromStr;

use serde_core::de::{self, Visitor};
use serde_core::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Timedelta, Timestamp};

/// Serializes `$type` as the string its `Display` and `FromStr` agree on.
macro_rules! spelled {
    ($type:ident, $visitor:ident, $expecting:literal) => {
        /// Reads the value from a string.
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
                deserializer.deserialize_str($visitor)
            }
        }

        impl Serialize for $type {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }
    };
}

spelled!(Timedelta, TimedeltaVisitor, "a span like \"5m\", \"250ms\" or \"0s\"");
spelled!(
    Timestamp,
    TimestampVisitor,
    "an RFC 3339 instant in UTC like \"2026-09-16T07:45:35.123Z\""
);

#[cfg(test)]
mod tests {
    use crate::{Timedelta, Timestamp};

    #[test]
    fn a_span_and_an_instant_round_trip_as_their_spellings() {
        let span = Timedelta::from_mins(5);
        assert_eq!(serde_json::to_string(&span).expect("a string"), r#""5m""#);
        assert_eq!(serde_json::from_str::<Timedelta>(r#""5m""#).expect("a span"), span);

        let instant = Timestamp::from_nanos(1_789_544_735_123_456_789);
        let text = r#""2026-09-16T07:45:35.123456789Z""#;
        assert_eq!(serde_json::to_string(&instant).expect("a string"), text);
        assert_eq!(serde_json::from_str::<Timestamp>(text).expect("an instant"), instant);
    }

    #[test]
    fn a_bare_number_is_not_a_time() {
        assert!(serde_json::from_str::<Timedelta>("300").is_err(), "a span names its unit");
        assert!(serde_json::from_str::<Timestamp>("1700000300").is_err(), "and so does an instant");
    }
}
