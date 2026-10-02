//! How a time crosses to a config file, a log or a peer.
//!
//! A [`Timestamp`](crate::Timestamp) and a [`Timedelta`] serialize as the strings
//! their `Display` writes: RFC 3339 for an instant, `"5m"` for a span. A peer that sends a count
//! instead names its unit at the field, and the counted type picks the module:
//!
//! ```
//! use serde::{Deserialize, Serialize};
//! use t2t_core::{Timedelta, Timestamp};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Listing {
//!     #[serde(with = "t2t_core::serde::timestamp::secs")]
//!     expiry: Timestamp,
//!     #[serde(with = "t2t_core::serde::timedelta::millis")]
//!     poll: Timedelta,
//! }
//!
//! let listing: Listing = serde_json::from_str(r#"{"expiry":1700000300,"poll":250}"#)?;
//! assert_eq!(listing.expiry, Timestamp::from_secs(1_700_000_300), "whole seconds");
//! assert_eq!(listing.poll, Timedelta::from_millis(250), "whole milliseconds");
//! # Ok::<(), serde_json::Error>(())
//! ```
//!
//! Each unit has an `option` module for a field that may be absent. Every module reads a count as
//! a number or a decimal string, refusing one past what the type holds, and writes a number, but
//! for nanoseconds since the epoch: those pass what an `f64` holds exactly, so a JSON reader backed
//! by one would round them, and they are written as a string.

#[cfg(feature = "schemars")]
mod schema;
mod spelling;

use core::fmt;

use serde_core::de::{self, Visitor};
use serde_core::{Deserializer, Serializer};

use crate::{OutOfRangeError, Timedelta};

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
        i64::try_from(count).ok().ok_or_else(|| E::custom(OutOfRangeError))
    }

    fn visit_str<E: de::Error>(self, count: &str) -> Result<i64, E> {
        count.parse().map_err(E::custom)
    }
}

/// Reads a count of `unit`s, whichever way it was written, as nanoseconds.
fn read<'de, D: Deserializer<'de>>(deserializer: D, unit: Timedelta) -> Result<i64, D::Error> {
    let count = deserializer.deserialize_any(CountVisitor)?;
    count.checked_mul(unit.as_nanos()).ok_or_else(|| de::Error::custom(OutOfRangeError))
}

/// Writes a count as a number.
fn write_number<S: Serializer>(count: i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(count)
}

/// Writes a count as a decimal string.
fn write_decimal<S: Serializer>(count: i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&count)
}

/// A `#[serde(with = …)]` module, and its `option` twin, for one type counted in one unit.
macro_rules! unit {
    ($module:ident, $type:ident, $unit:ident, $as_unit:ident, $write:ident, $doc:literal) => {
        #[doc = concat!("A [`", stringify!($type), "`](crate::", stringify!($type), ") as ", $doc, ".")]
        pub mod $module {
            use serde_core::{Deserializer, Serializer};

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
                read(deserializer, crate::Timedelta::$unit).map($type::from_nanos)
            }

            /// The same, for a field that may be absent.
            pub mod option {
                use serde_core::{Deserialize, Deserializer, Serializer};

                use crate::serde::{read, $write};
                use crate::$type;

                /// Writes the count, or nothing.
                ///
                /// # Errors
                /// Whatever the serializer reports.
                pub fn serialize<S: Serializer>(
                    value: &Option<$type>, serializer: S,
                ) -> Result<S::Ok, S::Error> {
                    match *value {
                        Some(value) => $write(value.$as_unit(), serializer),
                        None => serializer.serialize_none(),
                    }
                }

                /// Reads the count, or nothing.
                ///
                /// # Errors
                /// Whatever the deserializer reports, a string that is not a count, or a count
                /// past what the type holds.
                pub fn deserialize<'de, D: Deserializer<'de>>(
                    deserializer: D,
                ) -> Result<Option<$type>, D::Error> {
                    /// A count, in a shape `Option` reads.
                    struct Counted($type);

                    impl<'de> Deserialize<'de> for Counted {
                        fn deserialize<R: Deserializer<'de>>(
                            deserializer: R,
                        ) -> Result<Self, R::Error> {
                            read(deserializer, crate::Timedelta::$unit)
                                .map($type::from_nanos)
                                .map(Self)
                        }
                    }

                    Ok(Option::<Counted>::deserialize(deserializer)?.map(|counted| counted.0))
                }
            }
        }
    };
}

/// A [`Timestamp`](crate::Timestamp) as a count since the Unix epoch, in the unit the module
/// names.
pub mod timestamp {
    unit!(secs, Timestamp, SECOND, as_secs, write_number, "whole seconds since the Unix epoch");
    unit!(
        millis,
        Timestamp,
        MILLISECOND,
        as_millis,
        write_number,
        "whole milliseconds since the Unix epoch"
    );
    unit!(
        micros,
        Timestamp,
        MICROSECOND,
        as_micros,
        write_number,
        "whole microseconds since the Unix epoch"
    );
    unit!(
        nanos,
        Timestamp,
        NANOSECOND,
        as_nanos,
        write_decimal,
        "nanoseconds since the Unix epoch, in a string"
    );
}

/// A [`Timedelta`] as a count, in the unit the module names.
pub mod timedelta {
    unit!(secs, Timedelta, SECOND, as_secs, write_number, "a count of whole seconds");
    unit!(millis, Timedelta, MILLISECOND, as_millis, write_number, "a count of whole milliseconds");
    unit!(micros, Timedelta, MICROSECOND, as_micros, write_number, "a count of whole microseconds");
    unit!(nanos, Timedelta, NANOSECOND, as_nanos, write_number, "a count of nanoseconds");
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString as _;

    use serde::{Deserialize, Serialize};

    use crate::{Timedelta, Timestamp};

    /// One field for each kind of module.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Fields {
        /// Whole seconds.
        #[serde(with = "crate::serde::timestamp::secs")]
        expiry: Timestamp,
        /// Nanoseconds, written as a string.
        #[serde(with = "crate::serde::timestamp::nanos")]
        exact: Timestamp,
        /// A span counted in milliseconds.
        #[serde(with = "crate::serde::timedelta::millis")]
        poll: Timedelta,
        /// A span that may be absent.
        #[serde(with = "crate::serde::timedelta::nanos::option")]
        budget: Option<Timedelta>,
    }

    /// The fields every test reads and writes.
    const FIELDS: Fields = Fields {
        expiry: Timestamp::from_secs(1_700_000_300),
        exact: Timestamp::from_nanos(1_789_544_735_123_456_789),
        poll: Timedelta::from_millis(250),
        budget: Some(Timedelta::from_nanos(500)),
    };

    /// Whether reading `expiry` through `timestamp::secs` fails with a message opening `message`.
    fn is_refused(expiry: &str, message: &str) -> bool {
        let document = format!(r#"{{"expiry":{expiry},"exact":"0","poll":0,"budget":null}}"#);
        serde_json::from_str::<Fields>(&document)
            .is_err_and(|error| error.to_string().starts_with(message))
    }

    #[test]
    fn each_unit_round_trips_as_its_count() {
        let written = serde_json::to_string(&FIELDS).expect("a document");
        assert_eq!(
            written,
            r#"{"expiry":1700000300,"exact":"1789544735123456789","poll":250,"budget":500}"#
        );
        assert_eq!(serde_json::from_str::<Fields>(&written).expect("the fields"), FIELDS);
    }

    #[test]
    fn a_count_is_read_quoted_or_bare() {
        let read: Fields = serde_json::from_str(
            r#"{"expiry":"1700000300","exact":1789544735123456789,"poll":"250","budget":null}"#,
        )
        .expect("the fields");
        assert_eq!(read, Fields { budget: None, ..FIELDS });
    }

    #[test]
    fn a_count_past_the_range_or_not_a_count_is_refused() {
        assert!(is_refused("253402300799", "out of range error"), "the year 9999");
        assert!(is_refused(r#""soon""#, "invalid digit"), "not a count");
        assert!(is_refused("18446744073709551615", "out of range error"), "past an `i64`");
    }
}
