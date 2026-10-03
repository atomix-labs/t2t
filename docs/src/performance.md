# Performance

Every figure on this page is from a run committed under a crate's
`benches/results/`: a directory of two runs' transcripts and a `manifest.toml`
that says how they were taken. Each run was taken on an AWS Graviton4 (Arm
Neoverse V2, a c8g.8xlarge) under Linux 6.12.53, built by rustc 1.101.0-nightly
of 2026-09-27 in the bench profile (release's settings, fat LTO, one codegen
unit) unless a row says it ran without LTO, pinned with `taskset` to a core the
kernel keeps other work off. Each figure is a median of 100 samples, and each
pair a run's two medians.

| Run                                                  | Crate     | What it compares                                                                                          |
| ---------------------------------------------------- | --------- | --------------------------------------------------------------------------------------------------------- |
| `2026-10-02T23-14Z-9776c95-span-divisors`            | t2t-core  | a span's spelling with each unit divided by a constant, against dividing by a unit read from a table      |
| `2026-10-03T08-54Z-635b659-calendar-and-digit-pairs` | t2t-core  | `#[inline]` on the calendar's conversions with and without LTO, and the digit-pair table against `write!` |
| `2026-10-03T10-21Z-8aeae1f-clock-reads`              | t2t-clock | one reading of each clock                                                                                 |

The name is the time the run was taken, in UTC, the commit it measured, and what
it measured.

## Reading a Clock

One reading, on one thread, in the clock-reads run:

| Clock                  | Nanoseconds  |
| ---------------------- | ------------ |
| `CoarseSystemClock`    | 9.849, 9.836 |
| `CoarseMonotonicClock` | 9.897, 9.891 |
| `Counter`              | 11.82, 11.82 |
| `RawMonotonicClock`    | 33.16, 33.16 |
| `MonotonicClock`       | 33.18, 33.18 |
| `BootClock`            | 33.19, 33.19 |
| `TaiClock`             | 33.19, 33.19 |
| `SystemClock`          | 33.21, 33.19 |
| `ThreadCpuClock`       | 272.8, 269.3 |
| `ProcessCpuClock`      | 394.2, 387.7 |

The bench reads each clock on 1, 2, 4 and 8 threads at once, so that a clock
whose reading shares a cache line with another core's would show the rise. This
run pinned every thread to one core, so its rows for more threads time the
threads taking turns, and show nothing of what cores cost each other; the
manifest has them.

The counter's read is one instruction, `mrs` of `cntvct_el0` on `aarch64` and
`rdtsc` on `x86_64`, in inline assembly that tells the compiler it touches no
memory, no stack and no flags. `now` is `#[inline(always)]`, since a call around
the read would move it. The coarse clocks hand back a value the kernel's timer
last wrote, without reading the counter, and cost the least; the CPU-time clocks
take a system call.

## Converting a Value

Per 1,024 values, varied over the range, in microseconds:

| Conversion                      | Microseconds per 1,024 | Run                            |
| ------------------------------- | ---------------------- | ------------------------------ |
| ticks to nanoseconds, at 24 MHz | 1.206, 1.207           | span divisors, after           |
| nanoseconds to ticks, at 24 MHz | 1.034, 1.034           | span divisors, after           |
| `Timestamp::to_utc`             | 0.9816, 0.9818         | calendar, fat LTO, `#[inline]` |
| `UtcDateTime::to_timestamp`     | 5.184, 5.184           | calendar, fat LTO, `#[inline]` |
| writing a `Timestamp`           | 24.22, 24.24           | calendar, fat LTO, `#[inline]` |
| reading a `Timestamp`           | 20.46, 20.44           | calendar, fat LTO, `#[inline]` |
| writing a `Timedelta`           | 46.27, 46.86           | span divisors, after           |
| reading a `Timedelta`           | 31.33, 30.74           | span divisors, after           |

Each row is measured at the commit its run names; the code of each path is the
variant the row names, the one the crates have.

## How Each Path Was Chosen

### No Division by a Number Known at Run Time

On every path a reading or a spelling takes, a division divides by a constant,
which the compiler turns into a multiply. A [`TickRate`][t2t::TickRate] works
out its factors once, so a conversion is a multiply and a shift, as
[The CPU Counter](the-cpu-counter.md#the-conversion) shows. `Timestamp::to_utc`
counts the seconds from the first day a timestamp reaches, so every split of
them is an unsigned division by a constant, the time of day's in 32 bits.

A span's spelling writes each unit through a function generic over the unit,
`write_unit::<UNIT>`, so each unit's division is by a constant: 46.27 and 46.86
µs per 1,024 spans, against 51.59 and 51.83 µs dividing by a unit read from a
table of the seven, a tenth faster.

### The Digit-Pair Table

A timestamp's date and time of day are written two digits at a time, from a
table of the hundred numbers below 100 as ASCII. Writing 1,024 instants takes
24.22 and 24.23 µs through the table, and 108.8 and 108.5 µs through `write!`'s
`{:02}`, four and a half times as long, under fat LTO; without LTO, 29.15 and
29.15 µs against 114.4 and 114.4.

### Inlining Across Crates

The calendar's conversions are `#[inline]`, so a build without link-time
optimization inlines them across crates. That is the build a dependency gets in
a default release profile, which the run stood in for with
`CARGO_PROFILE_BENCH_LTO=false` and 16 codegen units. The run timed the
conversions with `#[inline]` on the calendar's five functions and without it,
each with fat LTO and without:

| Per 1,024, µs               | fat LTO, plain | fat LTO, `#[inline]` | no LTO, plain | no LTO, `#[inline]` |
| --------------------------- | -------------- | -------------------- | ------------- | ------------------- |
| `Timestamp::to_utc`         | 0.9875, 0.9851 | 0.9816, 0.9818       | 8.009, 8.002  | 0.9788, 0.9786      |
| `UtcDateTime::to_timestamp` | 4.559, 4.568   | 5.184, 5.184         | 6.912, 6.912  | 5.091, 5.087        |
| reading a `Timestamp`       | 20.58, 20.60   | 20.46, 20.44         | 29.03, 29.03  | 19.51, 19.50        |

Without LTO, `#[inline]` makes `to_utc` eight times as fast, and reading a
timestamp half as fast again. Under fat LTO, which inlines across crates anyway,
`to_utc` gains nothing, and `to_timestamp` is 13% slower for it, 5.18 µs against
4.56. Per conversion, a fat-LTO build pays 0.6 ns in `to_timestamp` for it, and
a build without LTO saves 6.9 ns in `to_utc`, 1.8 ns in `to_timestamp` and 9.3
ns in reading a timestamp. The crates keep `#[inline]`, for the build without
LTO.

## Measuring on Your Own Machine

The benches are divan's, one for t2t-core's conversions and one for the clocks'
reads:

```sh
cargo bench -p t2t-core --bench convert
cargo bench -p t2t-clock --features std --bench read
```

A name after `--` runs the benches it matches, as `-- timestamp`. For figures
that hold still, build first, so the run times no compilation, then pin the run
to a core the kernel keeps other work off, one that `isolcpus` and `nohz_full`
name on the kernel's command line, and run it twice:

```sh
cargo bench -p t2t-clock --features std --bench read --no-run
taskset -c 10 cargo bench -p t2t-clock --features std --bench read
```

The clock tour prints a rough figure for every clock in one command, timing
100,000 readings of each on the counter:

```sh
cargo run --release -p t2t --features std --example clock-tour
```

A run worth keeping goes under the crate's `benches/results/`, in a directory
named for its time in UTC, its commit and what it measures, with its transcripts
and a `manifest.toml` in the shape of those there: the hardware, the
environment, the toolchain, the parameters, and the results.
