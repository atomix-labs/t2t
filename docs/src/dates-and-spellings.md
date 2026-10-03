# Dates and Spellings

Every value has one spelling: `Display` writes it, `FromStr` reads it back, and
[Serialization](serialization.md) takes the same text. A timestamp's spelling is
RFC 3339 in UTC, read off the calendar view this chapter starts with.

## A Timestamp as a Date

`Timestamp::to_utc` reads a point as a [`UtcDateTime`][t2t::UtcDateTime]: its
year, month and day in the proleptic Gregorian calendar, its hour, minute and
second in UTC, and the nanoseconds past the second. Unix time has no leap
seconds, so the second is never 60.

A `UtcDateTime` is a view: arithmetic stays on the `Timestamp` it came from. Its
fields are public, so one may be built by hand; `is_valid` checks that every
field names a real date and time, and `to_timestamp` gives the point back, or
`None` for one that is not valid or is outside 1677-09-21 to 2262-04-11.

```rs
{{#include ../../crates/t2t/tests/book/dates_and_spellings.rs:calendar}}
```

## RFC 3339

A `Timestamp` is written as RFC 3339 in UTC, with `Z`, and nine fraction digits:
`2026-09-16T07:45:35.123456789Z`. A precision chooses the digits: `{:.3}` writes
milliseconds, `{:.0}` none, and nine is the most. A width pads the whole
spelling, with the fill and alignment given, and never cuts it. `Debug` writes
what `Display` does.

```rs
{{#include ../../crates/t2t/tests/book/dates_and_spellings.rs:write}}
```

`FromStr` reads what `Display` writes: `T` and `Z` in either case, and a
fraction of one to nine digits, or none. It refuses an offset, a 60th second, a
date that does not exist, an instant outside the range, and any text after the
`Z`.

```rs
{{#include ../../crates/t2t/tests/book/dates_and_spellings.rs:read}}
```

## TAI

A [`TaiTimestamp`][t2t::TaiTimestamp] is written as RFC 3339's date and time,
then the zone ` TAI`: `2026-09-16T07:46:12.123456789 TAI`. The precision and the
width work as they do for a `Timestamp`, and `FromStr` reads the zone back, and
refuses a `Z`, since an instant in UTC is not one in TAI.

```rs
{{#include ../../crates/t2t/tests/book/dates_and_spellings.rs:tai}}
```

## Spans

A [`Timedelta`][t2t::Timedelta] is written the way a config spells a span:
counts with units, coarsest first, from `d`, `h`, `m`, `s`, `ms`, `us` and `ns`.
Once a unit is written, every finer one with a remainder is too, a zero among
them, so a minute and 123 ms is `1m0s123ms`. Zero is `0s`, and a span backwards
starts with `-`. The longest spelling is `Timedelta::MIN`'s,
`-106751d23h47m16s854ms775us808ns`.

`FromStr` reads that, the bare `0`, and any count in each unit, so `48h` reads
as the span `2d` spells. It takes each unit at most once and coarsest first, and
refuses a span past the range.

An `Uptime`, a `RawUptime` and a `BootUptime` are written as the span since
their origin, and read back from it.

```rs
{{#include ../../crates/t2t/tests/book/dates_and_spellings.rs:spans}}
```

## Counts

A [`Tickstamp`][t2t::Tickstamp] and a [`Tickdelta`][t2t::Tickdelta] are written
as their count, then ` ticks`: `24 ticks`, `-24 ticks`. A
[`TickRate`][t2t::TickRate] is written as its count of hertz, then ` Hz`:
`24000000 Hz`. A width pads the count and its unit together. `FromStr` reads
what `Display` writes, and nothing else: no `+`, no `1 tick`, no `kHz`, and no
rate of zero.

```rs
{{#include ../../crates/t2t/tests/book/dates_and_spellings.rs:counts}}
```

## Refusals

Each spelling has its refusal, and values that share a spelling share it:

| Refusal                                                 | Refuses the text of                                       |
| ------------------------------------------------------- | --------------------------------------------------------- |
| [`ParseTimestampError`][t2t::ParseTimestampError]       | a `Timestamp`                                             |
| [`ParseTaiTimestampError`][t2t::ParseTaiTimestampError] | a `TaiTimestamp`                                          |
| [`ParseTimedeltaError`][t2t::ParseTimedeltaError]       | a `Timedelta`, an `Uptime`, a `RawUptime`, a `BootUptime` |
| [`ParseTickdeltaError`][t2t::ParseTickdeltaError]       | a `Tickdelta`, a `Tickstamp`                              |
| [`ParseTickRateError`][t2t::ParseTickRateError]         | a `TickRate`                                              |

Each is a unit struct, whose message names the spelling it expected, with an
example of it.

```rs
{{#include ../../crates/t2t/tests/book/dates_and_spellings.rs:refusals}}
```
