# Introduction

t2t is a set of Rust crates for time: the points a program reads off its clocks,
the spans between them, and the clocks themselves. Each timeline has a point
type of its own, so a reading of the wall clock is never subtracted from a
reading of the monotonic clock: the compiler refuses it.

It is for any program that needs time values or clocks: a server's deadlines, a
log's stamps, a benchmark's spans, a test that moves time by hand. It is no
trading library, though it is built for code where a nanosecond counts: every
point and span is one `i64`, the CPU's counter is read in one instruction, and
no conversion on a reading's or a spelling's path divides by a number known only
at run time.

## What It Holds

- Points, one per timeline: [`Timestamp`][t2t::Timestamp] on the wall clock,
  [`TaiTimestamp`][t2t::TaiTimestamp] on International Atomic Time,
  [`Uptime`][t2t::Uptime], [`RawUptime`][t2t::RawUptime] and
  [`BootUptime`][t2t::BootUptime] on the clocks that count from boot, and
  [`Tickstamp`][t2t::Tickstamp] on the CPU's counter.
- Spans, one per kind of count: [`Timedelta`][t2t::Timedelta] counts nanoseconds
  and [`Tickdelta`][t2t::Tickdelta] a counter's ticks; a
  [`TickRate`][t2t::TickRate] turns one into the other.
- Two views: [`UtcDateTime`][t2t::UtcDateTime] reads a timestamp as a date and a
  time of day, and [`Timed`][t2t::Timed] holds a value with the stamp it was
  captured with.
- Clocks, each of which implements [`Clock`][t2t::clock::Clock], whose one verb,
  `now`, gives a point on the clock's timeline, or a span for a clock of CPU
  time.

A point minus a point is a span, a point plus a span is a point, and every
operator saturates at the ends of the range, with a `checked_*` twin. Every
value has one spelling, which `Display` writes and `FromStr` reads back.

## The Crates

| Crate       | What it holds                                                        |
| ----------- | -------------------------------------------------------------------- |
| `t2t`       | everything, the values at its root and the clocks under `t2t::clock` |
| `t2t-core`  | the values; `no_std`, and never reaches the operating system         |
| `t2t-clock` | the clocks: the operating system's, the CPU's counter, manual clocks |

A program needs [`t2t`] alone; the other two are for a crate that wants the
values without the clocks, or the clocks without the facade.

## What It Leaves Out

- Time zones beyond UTC and TAI. A timestamp is written and read in UTC, and a
  spelling with an offset is refused. A zone or an offset from chrono, jiff or
  time converts as the instant it names.
- Calendars beyond the proleptic Gregorian. A date is a day of that calendar in
  UTC, from 1677-09-21 to 2262-04-11, with no leap seconds, as Unix time has
  none.
- Formatting beyond RFC 3339 and its own spellings. There are no format patterns
  and no names of months or days.

[How t2t Compares](how-t2t-compares.md) names the crates that do these, and
[Working with Other Crates](working-with-other-crates.md) shows the conversions
to and from them.

## How to Read This Book

The guide teaches the pieces in the order a program meets them, from a first
program to testing with a manual clock; the reference holds what each path
costs, what builds where, and how t2t compares with its neighbours. The API
documentation, under API Reference in the menu bar, holds each item's detail.

Every listing on these pages is part of a test that cargo builds and runs, so
the code shown compiles against the crates as they are. Most are under
`crates/t2t/tests/book/`; those that need `serde_json` or another crate's types
are under `crates/t2t-core/tests/book/`, and name the crate `t2t_core`, whose
items the facade re-exports under the same names.
