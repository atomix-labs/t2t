# Platforms and Features

The values build without `std`, bare metal included; the CPU's counter builds on
`aarch64` and `x86_64`; and the operating system's clocks build with `std`, on
64-bit Linux and macOS.

## What Builds Where

| Part                                | Builds                                                          |
| ----------------------------------- | --------------------------------------------------------------- |
| the values, t2t-core                | without `std`, bare metal included                              |
| `ManualClock`                       | without `std`, bare metal included                              |
| `AtomicManualClock`                 | on a target with 64-bit atomics                                 |
| `Counter`                           | on `aarch64` and `x86_64`, on any system, with or without `std` |
| the operating system's clocks       | with `std`, on 64-bit Linux and macOS                           |
| `TaiClock`                          | with `std`, on 64-bit Linux                                     |
| the `SystemTime` conversions        | with `std`, on 64-bit Linux and macOS                           |
| an `x86_64` counter's measured rate | with `std`, on 64-bit Linux and macOS                           |

On another target the parts it cannot build are left out, and the rest builds.
The operating system's clocks read a 64-bit `timespec`, which is why they need
64-bit pointers. On `aarch64`, Linux and macOS let a program read the counter
and its rate; a system that does not traps the read, which stops the process. On
`x86_64` without `std`, or off 64-bit Linux and macOS, a counter whose rate
CPUID does not report is refused with `NoRate`, since there is no clock to
measure it against.

## Without `std`

Every crate is `#![no_std]`, and nothing reaches the operating system unless
`std` is on. t2t-core never reaches it at all: its `std` adds the `SystemTime`
conversions, and turns on its dependencies' own `std`. The `schemars` feature
needs `alloc`, for the schemas it builds. CI builds all three crates for
`aarch64-unknown-none` and `x86_64-unknown-none` with no feature, and the facade
there with every feature that needs no `std`: `chrono-04`, `jiff-02`,
`schemars`, `serde`, `time-03` and `zerocopy-08`.

## 32-Bit Linux

On 32-bit Linux the operating system's clocks and the `SystemTime` conversions
are left out. On x32, `x86_64` with 32-bit pointers, the counter builds, its
rate from CPUID alone; on `i686` there is no counter. CI checks every crate with
every feature on both.

## Crate Features

None is on by default, and nothing reaches the operating system unless `std` is
named; what `std` adds is on 64-bit Linux and macOS.

| Feature       | Adds                                                                         |
| ------------- | ---------------------------------------------------------------------------- |
| `std`         | the OS clocks, an `x86_64` counter's measured rate, `SystemTime` conversions |
| `serde`       | every value's spelling, and the `serde` modules for counts in a named unit   |
| `schemars`    | `JsonSchema` for every value with a spelling; turns `serde` on               |
| `zerocopy-08` | the zerocopy traits each type can honour, native-endian                      |
| `chrono-04`   | conversions to and from chrono 0.4's `DateTime` and `TimeDelta`              |
| `jiff-02`     | conversions to and from jiff 0.2's `Timestamp` and `SignedDuration`          |
| `time-03`     | conversions to and from time 0.3's `OffsetDateTime` and `Duration`           |

That is the facade's table. t2t-core has every feature but its `std` adds the
`SystemTime` conversions alone, and t2t-clock has `std` alone, which adds the OS
clocks and an `x86_64` counter's measured rate. Features are named with `cargo
add`, as `cargo add t2t --features std,serde`.

## Rust Version

Every crate builds on stable Rust 1.98 or later, the workspace's `rust-version`,
and uses no nightly feature. The repository pins a nightly toolchain for
rustfmt's and the lints' nightly options alone, and `just check-rust-msrv`
builds every crate on 1.98.

## What CI Checks

`check.yml` runs every `just check` recipe on x86_64 Linux. `platforms.yml` runs
what it cannot:

- the tests on arm64 Linux, and on macOS on arm64 and on x86_64;
- the bare-metal builds, and the 32-bit Linux checks, above;
- every combination of every crate's features, each crate built alone;
- the loom models of `AtomicManualClock`, which try every interleaving of two
  threads its orderings allow;
- the two parsers fuzzed, from the corpus committed under
  `crates/t2t-core/fuzz/`.
