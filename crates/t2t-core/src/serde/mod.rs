//! How a time crosses to a config file, a log or a peer.
//!
//! Every value serializes as the string its `Display` writes, where a person reads the format:
//! RFC 3339 for an instant, `"1m30s"` for a span, `"24 ticks"` for a count of ticks. A format no
//! person reads, such as bincode or postcard, takes each as its count. A peer that sends a count
//! in a format a person reads names the unit at the field, and the counted type picks the module.
//!
//! Each unit has an `option` module for a field that may be absent. Every module reads a count as
//! a number or a decimal string, refusing one past what the type holds, and writes a number, but
//! for nanoseconds since the epoch: those pass what an `f64` holds exactly, so a JSON reader backed
//! by one would round them, and they are written as a string.
//!
//! # Examples
//! ```
//! use serde::{Deserialize, Serialize};
//! use serde_json::from_str;
//! use t2t_core::serde::{timedelta, timestamp};
//! use t2t_core::{Timedelta, Timestamp};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Listing {
//!     #[serde(with = "timestamp::secs")]
//!     expiry: Timestamp,
//!     #[serde(with = "timedelta::millis")]
//!     poll: Timedelta,
//! }
//!
//! let listing: Listing = from_str(r#"{"expiry":1700000300,"poll":250}"#)?;
//! assert_eq!(listing.expiry, Timestamp::from_secs(1_700_000_300), "whole seconds");
//! assert_eq!(listing.poll, Timedelta::from_millis(250), "whole milliseconds");
//! # Ok::<(), serde_json::Error>(())
//! ```

#[cfg(feature = "schemars")]
mod schema;
mod spelling;

use core::fmt;

use serde_core::de::{self, Visitor};
use serde_core::{Deserializer, Serializer};

use crate::OutOfRangeError;
use crate::errors::narrow;

/// Reads a count written as a number or as a decimal string.
struct CountVisitor;

impl Visitor<'_> for CountVisitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a count, as a number or a decimal string")
    }

    fn visit_i64<E: de::Error>(self, count: i64) -> Result<i64, E> {
        Ok(count)
    }

    fn visit_u64<E: de::Error>(self, count: u64) -> Result<i64, E> {
        narrow(count).map_err(E::custom)
    }

    fn visit_str<E: de::Error>(self, count: &str) -> Result<i64, E> {
        count.parse().map_err(E::custom)
    }
}

/// Reads a count of units `nanos_per_unit` nanoseconds long, as nanoseconds: whichever way it was
/// written, where a person reads the format, and as the `i64` both writers write where none does,
/// since a format that does not describe itself, as bincode or postcard, reads only the type it is
/// asked for.
fn read<'de, D: Deserializer<'de>>(deserializer: D, nanos_per_unit: i64) -> Result<i64, D::Error> {
    let count = if deserializer.is_human_readable() {
        deserializer.deserialize_any(CountVisitor)?
    } else {
        deserializer.deserialize_i64(CountVisitor)?
    };
    count.checked_mul(nanos_per_unit).ok_or_else(|| de::Error::custom(OutOfRangeError))
}

/// Writes a count as a number.
fn write_number<S: Serializer>(count: i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(count)
}

/// Writes a count as a decimal string, or as a number where no person reads the format.
fn write_decimal<S: Serializer>(count: i64, serializer: S) -> Result<S::Ok, S::Error> {
    if serializer.is_human_readable() {
        serializer.collect_str(&count)
    } else {
        serializer.serialize_i64(count)
    }
}

/// A `#[serde(with = …)]` module, and its `option` twin, for one type counted in one unit.
macro_rules! count_module {
    (
        $module:ident,
        $type:ident,
        $as_unit:ident,
        $nanos_per_unit:ident,
        $write:ident,
        $doc:literal $(,)?
    ) => {
        #[doc = concat!("A [`", stringify!($type), "`](crate::", stringify!($type), ") as ", $doc, ".")]
        pub mod $module {
            use serde_core::{Deserializer, Serializer};

            use crate::consts::$nanos_per_unit;
            use crate::serde::{read, $write};
            use crate::$type;

            /// Writes the count.
            ///
            /// # Errors
            /// Whatever the serializer reports.
            pub fn serialize<S: Serializer>(
                value: &$type, serializer: S,
            ) -> Result<S::Ok, S::Error> {
                $write(value.$as_unit(), serializer)
            }

            /// Reads the count.
            ///
            /// # Errors
            /// Whatever the deserializer reports, a string that is not a count, or a count past
            /// what the type holds.
            pub fn deserialize<'de, D: Deserializer<'de>>(
                deserializer: D,
            ) -> Result<$type, D::Error> {
                read(deserializer, $nanos_per_unit).map($type::from_nanos)
            }

            /// The same, for a field that may be absent.
            pub mod option {
                use serde_core::{Deserialize, Deserializer, Serialize, Serializer};

                use crate::consts::$nanos_per_unit;
                use crate::serde::{read, $write};
                use crate::$type;

                /// A count, in a shape `Option` writes and reads, with the presence a format no
                /// person reads marks.
                struct Counted($type);

                impl Serialize for Counted {
                    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                        $write(self.0.$as_unit(), serializer)
                    }
                }

                impl<'de> Deserialize<'de> for Counted {
                    fn deserialize<D: Deserializer<'de>>(
                        deserializer: D,
                    ) -> Result<Self, D::Error> {
                        read(deserializer, $nanos_per_unit).map($type::from_nanos).map(Self)
                    }
                }

                /// Writes the count, or nothing.
                ///
                /// # Errors
                /// Whatever the serializer reports.
                pub fn serialize<S: Serializer>(
                    value: &Option<$type>, serializer: S,
                ) -> Result<S::Ok, S::Error> {
                    value.map(Counted).serialize(serializer)
                }

                /// Reads the count, or nothing.
                ///
                /// # Errors
                /// Whatever the deserializer reports, a string that is not a count, or a count
                /// past what the type holds.
                pub fn deserialize<'de, D: Deserializer<'de>>(
                    deserializer: D,
                ) -> Result<Option<$type>, D::Error> {
                    Ok(Option::<Counted>::deserialize(deserializer)?.map(|counted| counted.0))
                }
            }
        }
    };
}

/// A [`Timestamp`](crate::Timestamp) as a count since the Unix epoch, in the unit the module
/// names.
pub mod timestamp {
    count_module!(
        secs,
        Timestamp,
        as_secs,
        NANOS_PER_SECOND,
        write_number,
        "whole seconds since the Unix epoch",
    );
    count_module!(
        millis,
        Timestamp,
        as_millis,
        NANOS_PER_MILLISECOND,
        write_number,
        "whole milliseconds since the Unix epoch",
    );
    count_module!(
        micros,
        Timestamp,
        as_micros,
        NANOS_PER_MICROSECOND,
        write_number,
        "whole microseconds since the Unix epoch",
    );
    count_module!(
        nanos,
        Timestamp,
        as_nanos,
        NANOS_PER_NANOSECOND,
        write_decimal,
        "nanoseconds since the Unix epoch, in a string",
    );
}

/// A [`Timedelta`](crate::Timedelta) as a count, in the unit the module names.
pub mod timedelta {
    count_module!(
        secs,
        Timedelta,
        as_secs,
        NANOS_PER_SECOND,
        write_number,
        "a count of whole seconds",
    );
    count_module!(
        millis,
        Timedelta,
        as_millis,
        NANOS_PER_MILLISECOND,
        write_number,
        "a count of whole milliseconds",
    );
    count_module!(
        micros,
        Timedelta,
        as_micros,
        NANOS_PER_MICROSECOND,
        write_number,
        "a count of whole microseconds",
    );
    count_module!(
        nanos,
        Timedelta,
        as_nanos,
        NANOS_PER_NANOSECOND,
        write_number,
        "a count of nanoseconds",
    );
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;

    use rstest::rstest;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string};
    use serde_test::{Configure as _, Token, assert_tokens};

    use super::{timedelta, timestamp};
    use crate::{Timedelta, Timestamp};

    /// One field for each kind of module.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Fields {
        /// Whole seconds.
        #[serde(with = "timestamp::secs")]
        expiry: Timestamp,
        /// Nanoseconds, written as a string.
        #[serde(with = "timestamp::nanos")]
        exact: Timestamp,
        /// A span counted in milliseconds.
        #[serde(with = "timedelta::millis")]
        poll: Timedelta,
        /// A span that may be absent.
        #[serde(with = "timedelta::nanos::option")]
        budget: Option<Timedelta>,
    }

    /// The fields every test reads and writes.
    const FIELDS: Fields = Fields {
        expiry: Timestamp::from_secs(1_700_000_300),
        exact: Timestamp::from_nanos(1_789_544_735_123_456_789),
        poll: Timedelta::from_millis(250),
        budget: Some(Timedelta::from_nanos(500)),
    };

    /// The reason a count past what its type holds is refused for.
    const OUT_OF_RANGE: &str = "out of range error: the time is outside what the target type holds";

    #[test]
    fn each_unit_round_trips_as_its_count() {
        let written = to_string(&FIELDS).expect("a document");
        assert_eq!(
            written,
            r#"{"expiry":1700000300,"exact":"1789544735123456789","poll":250,"budget":500}"#,
            "each field as a count in its unit"
        );
        assert_eq!(from_str::<Fields>(&written).expect("the fields"), FIELDS, "and back");
    }

    #[test]
    fn a_format_no_person_reads_takes_each_count_as_an_integer() {
        assert_tokens(
            &FIELDS.compact(),
            &[
                Token::Struct { name: "Fields", len: 4 },
                Token::Str("expiry"),
                Token::I64(1_700_000_300),
                Token::Str("exact"),
                Token::I64(1_789_544_735_123_456_789),
                Token::Str("poll"),
                Token::I64(250),
                Token::Str("budget"),
                Token::Some,
                Token::I64(500),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn a_count_is_read_quoted_or_bare() {
        let read: Fields = from_str(
            r#"{"expiry":"1700000300","exact":1789544735123456789,"poll":"250","budget":null}"#,
        )
        .expect("the fields");
        assert_eq!(read, Fields { budget: None, ..FIELDS }, "each count, however it came");
    }

    #[rstest]
    #[case::the_year_9999("253402300799", OUT_OF_RANGE)]
    #[case::not_a_count(r#""soon""#, "invalid digit found in string")]
    #[case::past_an_i64("18446744073709551615", OUT_OF_RANGE)]
    fn a_count_past_the_range_or_not_a_count_is_refused(
        #[case] expiry: &str, #[case] reason: &str,
    ) {
        let document = format!(r#"{{"expiry":{expiry},"exact":"0","poll":0,"budget":null}}"#);
        let error = from_str::<Fields>(&document).expect_err("a count the field refuses");
        let written = error.to_string();
        assert_eq!(written.split(" at line ").next(), Some(reason), "refused for its reason");
    }
}
