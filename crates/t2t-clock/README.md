# `t2t-clock`

The clocks of [t2t][project]: the operating system's wall, monotonic, boot and
CPU-time clocks, read with `clock_gettime` on Linux and macOS; the CPU's own
counter, read in one instruction on `aarch64` and `x86_64`; and clocks set and
moved by hand, for tests and replays. Each answers one verb, `now`, with a value
of `t2t-core` whose type names its timeline. [`t2t`][t2t] holds both.

## Install

```sh
cargo add t2t-clock --features std
```

`t2t-clock` builds with Rust 1.98 or later. The operating system's clocks need
`std`, on 64-bit Linux or macOS; the CPU's counter and the manual clocks build
without it.

## Quick Start

```rust
use t2t_clock::{Clock, ManualClock};
use t2t_core::{Timedelta, Timestamp};

/// Whether the sample stamped at `stamp` is older than a second.
fn is_stale<C: Clock<Reading = Timestamp>>(clock: &C, stamp: Timestamp) -> bool {
    clock.now() - stamp > Timedelta::SECOND
}

fn main() {
    let clock = ManualClock::new(Timestamp::from_secs(10));
    assert!(!is_stale(&clock, Timestamp::from_secs(10)), "fresh at capture");
    clock.advance(Timedelta::from_secs(2));
    assert!(is_stale(&clock, Timestamp::from_secs(10)), "stale two seconds on");
}
```

## Features

| Feature | Adds                                                   |
| ------- | ------------------------------------------------------ |
| `std`   | the OS clocks, and an `x86_64` counter's measured rate |

None is on by default.

## Documentation

- [The book][book]: which clock to choose, and what each costs.
- [The API on docs.rs][docs.rs], with an example for each clock.
- [CHANGELOG.md][changelog]: what changed in each release.

## License

Either [the MIT License][mit] or [the Apache License, Version 2.0][apache], at
your option.

[project]: https://github.com/atomix-labs/t2t
[t2t]: https://crates.io/crates/t2t
[book]: https://atomix-labs.github.io/t2t/
[docs.rs]: https://docs.rs/t2t-clock
[changelog]: https://github.com/atomix-labs/t2t/blob/main/CHANGELOG.md
[mit]: https://github.com/atomix-labs/t2t/blob/main/LICENSE-MIT
[apache]: https://github.com/atomix-labs/t2t/blob/main/LICENSE-APACHE
