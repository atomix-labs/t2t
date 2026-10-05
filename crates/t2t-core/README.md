# `t2t-core`

The values of [t2t][project]: points on the wall, monotonic, boot and atomic
timelines, a hardware counter's readings, the spans between them, the rate that
turns ticks into nanoseconds, a UTC calendar, and a value stamped with the time
it was captured. Each is one `i64` or a few, `no_std` and allocating nothing,
and each has a spelling it writes and reads back. The clocks that read them are
in `t2t-clock`, and [`t2t`][t2t] holds both.

## Install

```sh
cargo add t2t-core
```

`t2t-core` builds with Rust 1.98 or later, with or without `std`.

## Quick Start

```rust
use t2t_core::{ParseTimedeltaError, Timed, Timedelta, Timestamp};

fn main() -> Result<(), ParseTimedeltaError> {
    let stale: Timedelta = "1s".parse()?;
    let captured = Timestamp::from_secs(1_700_000_000);
    let sample = Timed::new(captured, 101_u64);

    assert!(sample.elapsed(captured + Timedelta::from_millis(1_500)) > stale, "stale by now");
    Ok(())
}
```

## Features

| Feature       | Adds                                                                       |
| ------------- | -------------------------------------------------------------------------- |
| `std`         | `SystemTime` conversions                                                   |
| `serde`       | every value's spelling, and the `serde` modules for counts in a named unit |
| `schemars`    | `JsonSchema` for every value with a spelling; turns `serde` on             |
| `zerocopy-08` | the zerocopy traits each type can honour, native-endian                    |
| `chrono-04`   | conversions to and from chrono 0.4's `DateTime` and `TimeDelta`            |
| `jiff-02`     | conversions to and from jiff 0.2's `Timestamp` and `SignedDuration`        |
| `time-03`     | conversions to and from time 0.3's `OffsetDateTime` and `Duration`         |

None is on by default: `cargo add t2t-core --features serde`.

## Documentation

- [The book][book]: what t2t is, and how its pieces fit.
- [The API on docs.rs][docs.rs], with an example for each type.
- [CHANGELOG.md][changelog]: what changed in each release.

## License

Either [the MIT License][mit] or [the Apache License, Version 2.0][apache], at
your option.

[project]: https://github.com/atomix-labs/t2t
[t2t]: https://crates.io/crates/t2t
[book]: https://atomix-labs.github.io/t2t/
[docs.rs]: https://docs.rs/t2t-core
[changelog]: https://github.com/atomix-labs/t2t/blob/main/CHANGELOG.md
[mit]: https://github.com/atomix-labs/t2t/blob/main/LICENSE-MIT
[apache]: https://github.com/atomix-labs/t2t/blob/main/LICENSE-APACHE
