# Points and Spans

A point is a moment on one timeline, and a span is how far apart two points are.
Each timeline has a point type of its own, and each kind of count a span type of
its own, so a sum or a difference that mixes two timelines does not compile.

## A Type per Timeline

| Point                               | Timeline                                   | Origin                  | Span                          |
| ----------------------------------- | ------------------------------------------ | ----------------------- | ----------------------------- |
| [`Timestamp`][t2t::Timestamp]       | the wall clock, in UTC                     | 1970-01-01T00:00:00Z    | [`Timedelta`][t2t::Timedelta] |
| [`TaiTimestamp`][t2t::TaiTimestamp] | International Atomic Time                  | 1970-01-01T00:00:00 TAI | `Timedelta`                   |
| [`Uptime`][t2t::Uptime]             | the monotonic clock                        | near boot               | `Timedelta`                   |
| [`RawUptime`][t2t::RawUptime]       | the monotonic clock at the hardware's rate | near boot               | `Timedelta`                   |
| [`BootUptime`][t2t::BootUptime]     | the boot clock, suspensions counted        | boot                    | `Timedelta`                   |
| [`Tickstamp`][t2t::Tickstamp]       | a hardware counter                         | the hardware's own      | [`Tickdelta`][t2t::Tickdelta] |

Every point and span is an `i64` and nothing else: nanoseconds for all but the
counter's, whose unit is its tick. Each is `Copy`, ordered and hashable, and its
`Default` is zero. A count of nanoseconds holds about 292 years either side of
its origin, so a `Timestamp` holds 1677-09-21 to 2262-04-11.
[Choosing a Clock](choosing-a-clock.md) says which clock reads which point.

## Arithmetic

A point minus a point is a span, a point plus or minus a span is a point, and
spans add, subtract, negate, scale by an `i64` and sum. Two points do not add.

```rs
{{#include ../../crates/t2t/tests/book/points_and_spans.rs:arithmetic}}
```

## Two Timelines Never Mix

Two timelines differ by an amount that only the machine knows, and that moves: a
time service steps the wall clock and never the monotonic clock; TAI runs ahead
of UTC by the offset the kernel holds; the raw monotonic clock drifts from the
monotonic clock by whatever slewing a time service applies; the boot clock gains
every suspension; and a counter's origin is the hardware's own. A difference
taken across two of them would be a number with no meaning, so the types refuse
it. A wall-clock instant minus an uptime does not compile:

```rs
{{#include ../../crates/t2t-core/tests/compile_fail/a_timestamp_minus_an_uptime.rs}}
```

```text
{{#include ../../crates/t2t-core/tests/compile_fail/a_timestamp_minus_an_uptime.stderr:1:7}}
```

Nor does a counter's reading moved by nanoseconds, since a tick's length is the
counter's rate, which the type does not know:

```rs
{{#include ../../crates/t2t-core/tests/compile_fail/a_tickstamp_plus_a_timedelta.rs}}
```

```text
{{#include ../../crates/t2t-core/tests/compile_fail/a_tickstamp_plus_a_timedelta.stderr}}
```

Each of these is a fixture that t2t's tests compile, and that must fail to
compile. A program that does need to compare two timelines says how: a counter's
span crosses to nanoseconds through its rate, as
[The CPU Counter](the-cpu-counter.md) shows, and two clocks' counts compare as
plain `i64`s, as the TAI clock's listing in
[Choosing a Clock](choosing-a-clock.md#tai) does.

## Units

Each nanosecond point has `from_nanos`, `from_micros`, `from_millis` and
`from_secs`, and an `as_*` for each; a `Timedelta` adds `from_mins`,
`from_hours` and `from_days`, and a constant for each unit from `NANOSECOND` to
`DAY`, a day being 24 hours, with no calendar consulted. A counter's point and
span have `from_ticks` and `as_ticks`. A coarser unit's constructor saturates
where its count passes the range.

A `Timedelta`'s accessor counts whole units toward zero, as std's `Duration`
does, and `subsec_nanos`, `subsec_micros` and `subsec_millis` give the rest,
signed as the span is. A point's accessor counts the whole units since its
origin, so a point before the origin rounds down, to the unit it falls in.

```rs
{{#include ../../crates/t2t/tests/book/points_and_spans.rs:units}}
```

## Saturation and the Checked Twins

Every operator saturates at the ends of the range, so none overflows or panics,
and each has a `checked_*` twin that returns `None` where the operator would
saturate:

| Operator                | Checked twin    |
| ----------------------- | --------------- |
| `point + span`          | `checked_add`   |
| `point - span`          | `checked_sub`   |
| `point - point`         | `checked_since` |
| `span + span`           | `checked_add`   |
| `span - span`           | `checked_sub`   |
| `span * count`          | `checked_mul`   |
| `-span`                 | `checked_neg`   |
| `span.abs()`            | `checked_abs`   |
| none: a span has no `/` | `checked_div`   |

The operator suits code where the end of the range is out of reach, or where
stopping there is right; the twin suits code to which reaching it is a bug. A
span has no `/`: `checked_div` divides it, refusing zero parts, and `MIN` in
`-1`.

```rs
{{#include ../../crates/t2t/tests/book/points_and_spans.rs:saturating}}
```

A workspace that denies clippy's `arithmetic_side_effects` lint names each point
and span in its clippy configuration's `arithmetic-side-effects-allowed`, since
the lint cannot see that they saturate; t2t's own
[`clippy.toml`](https://github.com/atomix-labs/t2t/blob/main/clippy.toml) shows
the list.

## Floor and Ceil

`floor` and `ceil` take a point or a span to the nearest multiple of a unit at
or below it, or at or above it: the minute a trade falls in, or the bar it
closes. They work on both sides of the origin, take the unit's magnitude, and
leave the value as it is for a zero unit.

```rs
{{#include ../../crates/t2t/tests/book/points_and_spans.rs:floor}}
```

## Code Generic Over Points

[`TimePoint`][t2t::TimePoint] is what every point shares: its span type, `Span`,
the operators between the two, and `from_count` and `count`, the point as its
`i64`. Code generic over it works on every timeline, as the manual clocks and
`Timed::elapsed` do, and a type of your own that wraps a point may implement it
to join them.

```rs
{{#include ../../crates/t2t/tests/book/points_and_spans.rs:generic}}
```
