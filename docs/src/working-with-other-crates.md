# Working with Other Crates

A `Timestamp` and a `Timedelta` convert to and from std's time types and those
of chrono, jiff and time. Where every t2t value fits the other type, the
conversion is a `From`; the way back is a `TryFrom`, refusing with
[`OutOfRangeError`][t2t::OutOfRangeError] what t2t's range does not hold. The
other points, a `TaiTimestamp` and those counted from boot or by a counter, have
no counterpart in these crates, and no conversion.

| Other type                   | Feature     | Into it                   | From it                                             |
| ---------------------------- | ----------- | ------------------------- | --------------------------------------------------- |
| `core::time::Duration`       | none        | a `Timedelta` forwards    | a `Duration` up to `Timedelta::MAX`                 |
| `std::time::SystemTime`      | `std`       | every `Timestamp`         | a moment from 1677-09-21 to 2262-04-11              |
| chrono 0.4's `DateTime<Utc>` | `chrono-04` | every `Timestamp`         | a `DateTime` in any zone, within the range          |
| chrono 0.4's `TimeDelta`     | `chrono-04` | every `Timedelta`         | a `TimeDelta` within the range                      |
| jiff 0.2's `Timestamp`       | `jiff-02`   | every `Timestamp`         | a `Timestamp` within the range                      |
| jiff 0.2's `SignedDuration`  | `jiff-02`   | every `Timedelta`         | a `SignedDuration` within the range                 |
| time 0.3's `OffsetDateTime`  | `time-03`   | every `Timestamp`, in UTC | an `OffsetDateTime` at any offset, within the range |
| time 0.3's `Duration`        | `time-03`   | every `Timedelta`         | a `Duration` within the range                       |

A zone or an offset converts as the instant it names, read in UTC. Each feature
is named for the version of the crate it converts with, and with `std` on, it
turns on that crate's own `std` too. The listings are t2t-core's tests, which
have each crate as a dependency, so they name the crate `t2t_core`.

## `std`

A `Timedelta` crosses to std's `Duration` and back by `TryFrom` both ways: a
`Duration` never runs backwards, so a negative span is refused, and a
`Timedelta` holds about 292 years, so a longer `Duration` is refused. The
conversion is `core`'s, and needs no feature.

```rs
{{#include ../../crates/t2t-core/tests/book/working_with_other_crates.rs:duration}}
```

A `Timestamp` becomes a `SystemTime` with `From`, since a `SystemTime` on 64-bit
Linux and macOS reaches past both ends of a timestamp's range, and comes back
with `TryFrom`, refusing a moment outside it. These need `std`, on 64-bit Linux
or macOS.

```rs
{{#include ../../crates/t2t-core/tests/book/working_with_other_crates.rs:system-time}}
```

std's `Instant` has no conversion: it is opaque, and std gives no way to read
its count or build one from a count. On Linux and macOS it reads
`CLOCK_MONOTONIC` and `CLOCK_UPTIME_RAW`, the clocks `MonotonicClock` reads, so
code that would convert an `Instant` reads `MonotonicClock` in its place.

## `chrono`

chrono's years reach far past 1677 and 2262, so a `DateTime<Utc>` holds every
timestamp, and a `TimeDelta` every span; a `DateTime` in another zone comes back
as the instant it names.

```rs
{{#include ../../crates/t2t-core/tests/book/working_with_other_crates.rs:chrono}}
```

## `jiff`

jiff's `Timestamp` reaches the years -9999 to 9999, so it holds every timestamp,
and its `SignedDuration` every span.

```rs
{{#include ../../crates/t2t-core/tests/book/working_with_other_crates.rs:jiff}}
```

## `time`

time's `OffsetDateTime` takes every timestamp, at the UTC offset; one at another
offset comes back as the instant it names. Its `Duration` holds every span.

```rs
{{#include ../../crates/t2t-core/tests/book/working_with_other_crates.rs:time}}
```
