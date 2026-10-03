# Serialization

With the `serde` feature, every value serializes as its spelling where a person
reads the format, and as its count where none does. The modules under
[`t2t::serde`] read and write a count in a named unit, for a field that holds
one. The `schemars` feature adds each spelling's JSON Schema, and the `zerocopy`
feature reads and writes values as bytes.

This chapter's listings are t2t-core's tests, since they depend on `serde_json`,
`serde_test` and the other crates' types; they name the crate `t2t_core`, and
the facade has the same items under `t2t`.

## Two Forms

A serializer that says it is human-readable, as `serde_json` and TOML's do,
takes each value's spelling; one that does not, as bincode and postcard do,
takes its count:

| Value                                            | Where a person reads the format       | Where none does |
| ------------------------------------------------ | ------------------------------------- | --------------- |
| `Timestamp`                                      | `"2026-09-16T07:45:35.123456789Z"`    | `i64`           |
| `TaiTimestamp`                                   | `"2026-09-16T07:46:12.123456789 TAI"` | `i64`           |
| `Timedelta`, `Uptime`, `RawUptime`, `BootUptime` | `"1m30s"`                             | `i64`           |
| `Tickstamp`, `Tickdelta`                         | `"24 ticks"`                          | `i64`           |
| `TickRate`                                       | `"24000000 Hz"`                       | `u64`           |

Each is read from the form it is written in. A rate of zero is refused, as serde
refuses a zero for a `NonZeroU64`.

```rs
{{#include ../../crates/t2t-core/tests/book/serialization.rs:spelled}}
```

`serde_test` shows both forms, its `readable` and `compact` standing for the two
kinds of format:

```rs
{{#include ../../crates/t2t-core/tests/book/serialization.rs:compact}}
```

## Counts in a Named Unit

A feed or a peer that sends a count names its unit at the field, and the field's
`#[serde(with = …)]` names the module:

| Module                                                                        | Reads and writes                 |
| ----------------------------------------------------------------------------- | -------------------------------- |
| [`timestamp::secs`][t2t::serde::timestamp::secs], `millis`, `micros`          | whole units since the Unix epoch |
| [`timestamp::nanos`][t2t::serde::timestamp::nanos]                            | nanoseconds since the Unix epoch |
| [`timedelta::secs`][t2t::serde::timedelta::secs], `millis`, `micros`, `nanos` | a span's count in the unit       |

Each has an `option` module, for an `Option` field, as
`timedelta::secs::option`. Where a person reads the format, a module reads a
count as a number or as a decimal string; where none does, as an `i64`. It
writes a number, but for nanoseconds since the epoch where a person reads the
format: those pass what an `f64` holds exactly, so a JSON reader backed by one
would round them, and they are written as a string.

```rs
{{#include ../../crates/t2t-core/tests/book/serialization.rs:units}}
```

A count past what the type holds is refused, with the reason
[`OutOfRangeError`][t2t::OutOfRangeError] gives. Writing a coarser unit drops
what is finer: a timestamp is written as the unit it falls in, rounded down, and
a span as its whole units toward zero.

```rs
{{#include ../../crates/t2t-core/tests/book/serialization.rs:range}}
```

## JSON Schema

With `schemars`, which turns `serde` on, every value with a spelling implements
`JsonSchema`: an inlined string schema with a description and a pattern, and for
a `Timestamp` the format `date-time`. Each pattern takes what its parser reads,
which t2t's tests check on spellings the parser takes and on ones it refuses; a
date that does not exist, or a count past the range, matches the pattern and is
the parser's alone to refuse.

```rs
{{#include ../../crates/t2t-core/tests/book/json_schema.rs:schema}}
```

A span's pattern holds a lookahead, which JSON Schema's ECMA-262 regular
expressions read; Rust's `regex` crate refuses a lookahead, so a validator built
on it cannot use the pattern.

## Bytes

With `zerocopy`, each type derives the zerocopy traits it can honour. A type's
bytes are native-endian, and with the feature on, its layout is part of its API.

| Type                                  | Traits                                               | Left out, and why                                                                                                                                        |
| ------------------------------------- | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| every point, `Timedelta`, `Tickdelta` | `FromBytes`, `IntoBytes`, `Immutable`, `KnownLayout` | `Unaligned`: each is an 8-byte-aligned `i64`                                                                                                             |
| [`Timed<T, S>`][t2t::Timed]           | `FromBytes`, `Immutable`, `KnownLayout`              | `IntoBytes`: zerocopy's derive takes a generic struct only where every field is `Unaligned`, and no stamp is                                             |
| [`UtcDateTime`][t2t::UtcDateTime]     | `FromBytes`, `Immutable`, `KnownLayout`              | `IntoBytes`: three bytes of padding sit between its seconds and its nanoseconds                                                                          |
| [`TickRate`][t2t::TickRate]           | `Immutable`, `KnownLayout`                           | `FromBytes` and `TryFromBytes`: a derived check would test that the rate is not zero, and take factors that do not match it; a rate crosses as its hertz |

A `Timed<u64>` is still read from bytes, its stamp's then its value's, and any
sixteen bytes make a `UtcDateTime`, so one read from bytes is checked with
`is_valid`, or by `to_timestamp`, before it is used.

```rs
{{#include ../../crates/t2t-core/tests/book/bytes.rs:bytes}}
```
