<!-- >>> devset: project >>> -->
<!-- dprint-ignore-start -->

<h1 align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/atomix-labs/t2t/main/docs/src/media/logo-dark.svg">
    <img alt="t2t" src="https://raw.githubusercontent.com/atomix-labs/t2t/main/docs/src/media/logo-light.svg" height="56">
  </picture>
</h1>

<p align="center">Time values and the clocks that read them, for code where a nanosecond counts.</p>

<p align="center">
  <a href="https://github.com/atomix-labs/t2t/actions/workflows/check.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/atomix-labs/t2t/check.yml?branch=main&amp;style=flat-square&amp;label=check"></a>
  <a href="https://crates.io/crates/t2t"><img alt="crates.io" src="https://img.shields.io/crates/v/t2t?style=flat-square"></a>
  <a href="https://docs.rs/t2t"><img alt="docs.rs" src="https://img.shields.io/docsrs/t2t?style=flat-square"></a>
  <a href="https://atomix-labs.github.io/t2t/"><img alt="Book" src="https://img.shields.io/badge/book-read-blue?style=flat-square"></a>
  <a href="https://github.com/atomix-labs/devset"><img alt="managed with devset" src="https://img.shields.io/badge/managed_with-devset-0969da?style=flat-square&amp;logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAzMiAzMiI+PHRpdGxlPmRldnNldDwvdGl0bGU+PHBhdGggZmlsbD0iI2YwZjZmYyIgZD0ibTE2IDMgMTMgNi41TDE2IDE2IDMgOS41WiIvPjxwYXRoIGZpbGw9Im5vbmUiIHN0cm9rZT0iI2YwZjZmYyIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjIuNSIgZD0ibTMgMTYgMTMgNi41TDI5IDE2TTMgMjIuNSAxNiAyOWwxMy02LjUiLz48L3N2Zz4K"></a>
</p>

<!-- dprint-ignore-end -->
<!-- <<< devset: project <<< -->

t2t holds the time a program reads and carries: points on the wall, monotonic,
boot and atomic timelines, a hardware counter's readings, the spans between
them, and the clocks that read each. Every point and span is one `i64`, and
every timeline its own type, so a reading from one clock is never subtracted
from another's; every operator saturates rather than overflows, and every value
has a spelling it writes and reads back. Nothing reaches the operating system
unless the `std` feature asks.

| Crate                    | What it holds                                                        |
| ------------------------ | -------------------------------------------------------------------- |
| [`t2t`][t2t]             | everything, the values at its root and the clocks under `t2t::clock` |
| [`t2t-core`][t2t-core]   | the values: points, spans, rates, the calendar, `Timed`; `no_std`    |
| [`t2t-clock`][t2t-clock] | the clocks: the OS's, the CPU's counter, and clocks set by hand      |

## Install

```sh
cargo add t2t --features std
```

t2t builds with Rust 1.98 or later, on Linux and macOS, on `aarch64` and
`x86_64`; the values and the CPU's counter build without `std` too.

## Quick Start

```rust
use t2t::clock::{Clock, ManualClock};
use t2t::{Timed, Timedelta, Timestamp};

fn main() {
    let clock = ManualClock::new(Timestamp::from_secs(1_700_000_000));
    let quote = Timed::new(clock.now(), 101_u64);

    clock.advance(Timedelta::from_millis(1_500));
    assert!(quote.elapsed(clock.now()) > Timedelta::SECOND, "stale a second and a half on");
}
```

In a clone of the repository, `cargo run --release -p t2t --features std
--example clock-tour` prints every clock's reading, and what one costs on the
machine it runs on.

## Clocks

| Clock                  | Reads          | Steps?  | For                                      |
| ---------------------- | -------------- | ------- | ---------------------------------------- |
| `SystemClock`          | `Timestamp`    | yes     | a stamp other machines compare           |
| `CoarseSystemClock`    | `Timestamp`    | yes     | whether a heartbeat or expiry is due     |
| `TaiClock` (Linux)     | `TaiTimestamp` | yes     | a stamp on the timescale PTP keeps       |
| `MonotonicClock`       | `Uptime`       | never   | a deadline, a timeout                    |
| `CoarseMonotonicClock` | `Uptime`       | never   | a far deadline, polled often             |
| `RawMonotonicClock`    | `RawUptime`    | never   | a span no time service's slewing touches |
| `BootClock`            | `BootUptime`   | never   | a timeout that runs through a suspension |
| `ProcessCpuClock`      | `Timedelta`    | never   | the CPU time the process has used        |
| `ThreadCpuClock`       | `Timedelta`    | never   | the CPU time the calling thread has used |
| `Counter`              | `Tickstamp`    | never   | a stamp or a span in one instruction     |
| `ManualClock`          | any point      | by hand | a test or a replay on one thread         |
| `AtomicManualClock`    | any point      | by hand | a test or a replay shared across threads |

## Features

What `std` adds is on 64-bit Linux and macOS.

| Feature     | Adds                                                                         |
| ----------- | ---------------------------------------------------------------------------- |
| `std`       | the OS clocks, an `x86_64` counter's measured rate, `SystemTime` conversions |
| `serde`     | every value's spelling, and the `serde` modules for counts in a named unit   |
| `schemars`  | `JsonSchema` for every value with a spelling; turns `serde` on               |
| `zerocopy`  | the zerocopy traits each type can honour, native-endian                      |
| `chrono-04` | conversions to and from chrono 0.4's `DateTime` and `TimeDelta`              |
| `jiff-02`   | conversions to and from jiff 0.2's `Timestamp` and `SignedDuration`          |
| `time-03`   | conversions to and from time 0.3's `OffsetDateTime` and `Duration`           |

None is on by default: `cargo add t2t --features std,serde`.

## Documentation

- [The book][book]: what t2t is, how its pieces fit, and which clock to choose.
- The API on docs.rs, with an example for each type: [`t2t`][docs-t2t],
  [`t2t-core`][docs-t2t-core] and [`t2t-clock`][docs-t2t-clock].
- [CHANGELOG.md][changelog]: what changed in each release.

## Contributing

Issues and pull requests are welcome: read [CONTRIBUTING.md][contributing]
first, and report a vulnerability as [SECURITY.md][security] says.

## License

Either [the MIT License][mit] or [the Apache License, Version 2.0][apache], at
your option.

[t2t]: https://crates.io/crates/t2t
[t2t-core]: https://crates.io/crates/t2t-core
[t2t-clock]: https://crates.io/crates/t2t-clock
[book]: https://atomix-labs.github.io/t2t/
[docs-t2t]: https://docs.rs/t2t
[docs-t2t-core]: https://docs.rs/t2t-core
[docs-t2t-clock]: https://docs.rs/t2t-clock
[changelog]: https://github.com/atomix-labs/t2t/blob/main/CHANGELOG.md
[contributing]: https://github.com/atomix-labs/t2t/blob/main/CONTRIBUTING.md
[security]: https://github.com/atomix-labs/t2t/blob/main/SECURITY.md
[mit]: https://github.com/atomix-labs/t2t/blob/main/LICENSE-MIT
[apache]: https://github.com/atomix-labs/t2t/blob/main/LICENSE-APACHE
