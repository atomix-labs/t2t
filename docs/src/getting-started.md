# Getting Started

Add the crate, with the operating system's clocks:

```sh
cargo add t2t --features std
```

The values, the CPU's counter and the manual clocks need no feature, and build
without `std`; the operating system's clocks need `std`, on 64-bit Linux or
macOS. t2t builds on stable Rust 1.98 or later.
[Platforms and Features](platforms-and-features.md) lists every feature and what
it adds.

## A First Program

The wall clock stamps the moment, since its reading names a moment another
machine can name too; the monotonic clock times the pause, since no correction
to the wall clock moves it.

```rs
{{#include ../../crates/t2t/tests/book/os/getting_started.rs:first}}
```

`Timestamp`'s `{:.6}` writes RFC 3339 to the microsecond, and a span is written
coarsest unit first, as `20ms83us`. [Choosing a Clock](choosing-a-clock.md) says
which clock to read for what.

## Two Tours

In a checkout of the repository, two examples walk through the rest. The first
prints the values: points and spans, the calendar, a stamped value, and a
counter's rate.

```sh
cargo run -p t2t --example time-tour
```

The second prints every clock's reading, and what one costs on the machine it
runs on.

```sh
cargo run --release -p t2t --features std --example clock-tour
```

Every listing in this book runs as a test too:

```sh
cargo test -p t2t --all-features --test book
cargo test -p t2t-core --all-features --test book
```

From there, [`clock`][t2t::clock] lists every clock, and the crate's page,
[`t2t`], says which to choose for what.
