<!-- >>> devset: project >>> -->
<!-- dprint-ignore-start -->

<h1 align="center">t2t</h1>

<p align="center">A time, counter and clock library.</p>

<p align="center">
  <a href="https://github.com/atomix-labs/t2t/actions/workflows/check.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/atomix-labs/t2t/check.yml?branch=main&amp;style=flat-square&amp;label=check"></a>
  <a href="https://crates.io/crates/t2t"><img alt="crates.io" src="https://img.shields.io/crates/v/t2t?style=flat-square"></a>
  <a href="https://docs.rs/t2t"><img alt="docs.rs" src="https://img.shields.io/docsrs/t2t?style=flat-square"></a>
  <a href="https://atomix-labs.github.io/t2t/"><img alt="Book" src="https://img.shields.io/badge/book-read-blue?style=flat-square"></a>
</p>

<!-- dprint-ignore-end -->
<!-- <<< devset: project <<< -->

t2t holds the time a program reads and carries: instants on the wall, monotonic,
boot and atomic timelines, a hardware counter's readings, the spans between
them, and the clocks that read each. Every value is one `i64`, and every
timeline its own type, so a reading from one clock is never subtracted from
another's; every operator saturates rather than overflows. Nothing reaches the
operating system unless the `std` feature asks.

| Crate       | What it holds                                                        |
| ----------- | -------------------------------------------------------------------- |
| `t2t`       | everything, the values at its root and the clocks under `t2t::clock` |
| `t2t-core`  | the values: points, spans, rates, the calendar, `Timed`; `no_std`    |
| `t2t-clock` | the clocks: the system's, the CPU's counter, and clocks set by hand  |

## Install

Until the first release, from git:

```sh
cargo add t2t --git https://github.com/atomix-labs/t2t --features std
```

t2t builds with Rust 1.98 or later, on Linux and macOS, on `aarch64` and
`x86_64`; the values and the CPU's counter build without `std` too.

## Quick Start

```rust
use t2t::clock::{Clock, ManualClock};
use t2t::{Timed, Timedelta, Timestamp};

let clock = ManualClock::new(Timestamp::from_secs(1_700_000_000));
let quote = Timed::new(clock.now(), 101_u64);

clock.advance(Timedelta::from_millis(1_500));
assert!(quote.elapsed(clock.now()) > Timedelta::SECOND, "stale a second and a half on");
```

`cargo run --release -p t2t --features std --example clock-tour` prints every
clock's reading, and what one costs on the machine it runs on.

## Clocks

| Clock                  | Reads          | For                                         |
| ---------------------- | -------------- | ------------------------------------------- |
| `SystemClock`          | `Timestamp`    | stamping a capture other machines compare   |
| `CoarseSystemClock`    | `Timestamp`    | asking whether a heartbeat or expiry is due |
| `TaiClock` (Linux)     | `TaiTimestamp` | a stamp on the timescale PTP keeps          |
| `MonotonicClock`       | `Uptime`       | a deadline, a timeout                       |
| `CoarseMonotonicClock` | `Uptime`       | a far deadline, polled often                |
| `RawMonotonicClock`    | `RawUptime`    | a span no time service's slewing touches    |
| `BootClock`            | `BootTime`     | a timeout that runs on through a suspension |
| `ProcessCpuClock`      | `Timedelta`    | the CPU time the process has used           |
| `ThreadCpuClock`       | `Timedelta`    | the CPU time the calling thread has used    |
| `Counter`              | `Tick`         | a stamp or a span in one instruction        |
| `ManualClock`          | any point      | a test or a replay on one thread            |
| `AtomicManualClock`    | any point      | a test or a replay shared across threads    |

## Features

| Feature     | Adds                                                                                |
| ----------- | ----------------------------------------------------------------------------------- |
| `std`       | the system clocks, `SystemTime` conversions, an x86_64 counter's rate               |
| `serde`     | the string spellings, and modules for counts in a named unit                        |
| `schemars`  | `JsonSchema` for `Timestamp` and `Timedelta`; turns `serde` on                      |
| `zerocopy`  | `FromBytes`, `IntoBytes` and the rest, where each type can honour them              |
| `chrono-04` | `Timestamp` and `Timedelta` to and from chrono 0.4's `DateTime` and `TimeDelta`     |
| `jiff-02`   | `Timestamp` and `Timedelta` to and from jiff 0.2's `Timestamp` and `SignedDuration` |
| `time-03`   | `Timestamp` and `Timedelta` to and from time 0.3's `OffsetDateTime` and `Duration`  |

None is on by default: `cargo add t2t --git https://github.com/atomix-labs/t2t
--features std,serde`.

## Documentation

- [The book][book]: what t2t is, and how its pieces fit.
- [CHANGELOG.md][changelog]: what changed in each release.

## Contributing

Issues and pull requests are welcome: read [CONTRIBUTING.md][contributing]
first.

## License

Either [the MIT License][mit] or [the Apache License, Version 2.0][apache], at
your option.

[book]: https://atomix-labs.github.io/t2t/
[changelog]: CHANGELOG.md
[contributing]: CONTRIBUTING.md
[mit]: LICENSE-MIT
[apache]: LICENSE-APACHE
