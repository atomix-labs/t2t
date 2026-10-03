# How t2t Compares

t2t sits beside std's time types, two crates that read the CPU's counter, quanta
and minstant, and three calendar crates, chrono, jiff and time. Each section
says what the other does, from its own documentation, where it and t2t differ,
and where it is the better choice. The versions read are those docs.rs served on
2026-10-03: std 1.99, quanta 0.13.0, minstant 0.1.7, chrono 0.4.45, jiff 0.2.37
and time 0.3.55. Where a crate's documentation and its source disagree, the
section says so.

|                         | t2t                        | std                         | quanta                    | minstant       | chrono                                                                | jiff                                        | time                                  |
| ----------------------- | -------------------------- | --------------------------- | ------------------------- | -------------- | --------------------------------------------------------------------- | ------------------------------------------- | ------------------------------------- |
| Points                  | one type per timeline, six | `Instant`, `SystemTime`     | one `Instant`             | one `Instant`  | date and time types                                                   | date and time types                         | date and time types                   |
| A signed span           | `Timedelta`                | none                        | none                      | none           | `TimeDelta`                                                           | `SignedDuration`, `Span`                    | `Duration`                            |
| Reads the CPU's counter | on `aarch64` and `x86_64`  | no                          | on `aarch64` and `x86_64` | on x86 Linux   | no                                                                    | no                                          | no                                    |
| Time zones              | UTC and TAI                | none                        | none                      | none           | `Utc`, `Local`, fixed offsets; IANA zones through chrono-tz or tzfile | IANA zones                                  | UTC offsets                           |
| Years                   | 1677 to 2262               | not a calendar              | not a calendar            | not a calendar | about ±262,000                                                        | -9999 to 9999                               | ±9999, or ±999,999 with `large-dates` |
| `no_std`                | yes                        | `Duration` alone, in `core` | no                        | no             | yes                                                                   | yes, with fixed zones alone without `alloc` | mostly                                |

## `std::time`

std has [`Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html), a
monotonic clock's reading, which reads `CLOCK_MONOTONIC` on Unix and
`CLOCK_UPTIME_RAW` on Darwin;
[`SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html), the
wall clock's, which is not monotonic; and
[`Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html), an
unsigned span of up to about 584 billion years, with no `Display`. An `Instant`
is opaque: it compares with another and subtracts from one, but has no count to
read and no constructor from one. std has no signed span, no calendar and no
RFC 3339.

t2t's `MonotonicClock` reads the clocks `Instant` reads, and its `SystemClock`
the one `SystemTime` reads; beside them it has TAI, the raw and boot clocks, and
the counter, each a type, with signed spans, counts to read, spellings and
manual clocks.

std is the better choice where a program times spans inside one process and
hands them to std's own APIs, which take a `Duration`: it needs no dependency.
t2t's `Timedelta` converts to and from a `Duration`.

## `quanta`

[quanta](https://docs.rs/quanta/0.13.0/quanta/) reads the time-stamp counter on
`x86_64` and the system counter on `aarch64`, and falls back to the operating
system's clock where the counter is missing or unreliable. Its documentation
asks for a constant or an invariant time-stamp counter; its source requires an
invariant one, and `rdtscp`. It calibrates the counter against the reference
clock once a process, lazily, at the first `Clock::new` or `Instant::now`, in up
to 200 ms. It has one `Instant`, a count of nanoseconds; a cached
`Clock::recent` that an upkeep thread keeps current; and a mock clock,
`Clock::mock`. It converts a reading with an integer multiply and a shift, as
t2t does. It builds on Windows, the BSDs and WebAssembly as well as Linux and
macOS, and its source needs `std`.

t2t reads the counter's rate where the CPU reports one, and measures it only on
`x86_64` where it does not; it refuses a counter it cannot read as time where
quanta falls back. Its readings are the raw register, so two processes agree on
every one, and it builds without `std`.

quanta is the better choice for a program that runs on Windows, a BSD or
WebAssembly; that wants the operating system's clock taken in silently where the
counter cannot serve; or that reads the time often enough to want a reading an
upkeep thread caches.

## `minstant`

[minstant](https://docs.rs/minstant/0.1.7/minstant/) reads the time-stamp
counter on Linux on x86 and `x86_64`, calibrated as the process starts. Its
documentation says that elsewhere it falls back to coarse time; its source falls
back to `SystemTime`, the wall clock, unless its `fallback-coarse` feature is
on. It has one `Instant`, and an `Anchor` that turns a reading into nanoseconds
since the Unix epoch; `is_tsc_available` says which it reads. Its source
converts a reading with floating point, and needs `std`.

t2t reads the counter on `aarch64` too, and on macOS; it never falls back to
another clock, and converts with integers. It has no anchor from a counter's
reading to a `Timestamp`: a program that needs one takes both readings itself.

minstant is the better choice for a program on Linux on x86 that wants one
`Instant` that builds everywhere, falling back off x86 Linux, and readings
turned into Unix time by its `Anchor`.

## `chrono`

[chrono](https://docs.rs/chrono/0.4.45/chrono/) is a calendar crate: a
`DateTime` in a time zone, `Utc`, `Local` or a fixed offset, with the IANA zones
through chrono-tz or tzfile; naive dates and times without a zone; and a signed
`TimeDelta` of seconds and nanoseconds. It formats with strftime-style patterns,
RFC 3339 and RFC 2822, reaches about 262,000 years either side of the common
era, and has no monotonic clock of its own: its documentation points to std's
`Instant`.

chrono is the better choice for local time, time zones, dates far outside 1677
to 2262, and formatting to a pattern. t2t converts a `Timestamp` and a
`Timedelta` to and from chrono's `DateTime` and `TimeDelta`.

## `jiff`

[jiff](https://docs.rs/jiff/0.2.37/jiff/) is a calendar crate built around the
IANA time zone database, the system's copy or one it embeds where the system has
none. Its `Zoned` does arithmetic that follows daylight saving time, its `Span`
holds calendar units such as months, and its `Timestamp` and `SignedDuration`
are 96-bit counts of nanoseconds over the years -9999 to 9999. It reads and
writes RFC 3339, RFC 9557 and RFC 2822, and strftime-style patterns; it has
nearly all of this without `std`.

jiff is the better choice for anything that follows a time zone: local
deadlines, daylight saving time, calendar arithmetic in months and days. t2t
converts a `Timestamp` and a `Timedelta` to and from jiff's `Timestamp` and
`SignedDuration`.

## `time`

[time](https://docs.rs/time/0.3.55/time/) is a calendar crate of dates and
times, at a UTC offset or in UTC, with a signed `Duration` of seconds and
nanoseconds. It formats and parses with format descriptions of any layout, and
with RFC 3339, RFC 2822 and ISO 8601; it holds years ±9999, or ±999,999 with its
`large-dates` feature; and it is mostly `no_std`.

time is the better choice for dates and times at an offset, and for formats
beyond RFC 3339. t2t converts a `Timestamp` and a `Timedelta` to and from time's
`OffsetDateTime` and `Duration`.

## Where t2t Fits

t2t is for the time a program reads off its clocks and carries: every value one
`i64`, every timeline a type, every operator saturating, and the counter read in
one instruction. It has no time zones but UTC and TAI, no calendar arithmetic,
and no formatting beyond its spellings; a program that needs those reads its
clocks with t2t and converts to chrono, jiff or time at the edge.
