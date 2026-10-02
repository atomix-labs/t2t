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
            "description": "A signed span: \"0\", or counts with units from coarsest to finest of d, h, m, s, ms, us, ns, as \"250ms\" or \"1h30m\".",
            "pattern": "^-?(0|([0-9]+(d|h|m|s|ms|us|ns))+)$"
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
