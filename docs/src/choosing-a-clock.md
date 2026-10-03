# Choosing a Clock

Every clock implements [`Clock`][t2t::clock::Clock], whose one verb, `now`,
gives the clock's `Reading`: a point on its timeline, or a span for a clock of
CPU time. Choosing a clock is choosing the timeline its readings are on.

## The Clocks

| Clock                                                      | Reads          | Linux                      | macOS                      | Steps               |
| ---------------------------------------------------------- | -------------- | -------------------------- | -------------------------- | ------------------- |
| [`SystemClock`][t2t::clock::SystemClock]                   | `Timestamp`    | `CLOCK_REALTIME`           | `CLOCK_REALTIME`           | yes                 |
| [`CoarseSystemClock`][t2t::clock::CoarseSystemClock]       | `Timestamp`    | `CLOCK_REALTIME_COARSE`    | `CLOCK_REALTIME`           | yes                 |
| [`TaiClock`][t2t::clock::TaiClock]                         | `TaiTimestamp` | `CLOCK_TAI`                | none                       | with the wall clock |
| [`MonotonicClock`][t2t::clock::MonotonicClock]             | `Uptime`       | `CLOCK_MONOTONIC`          | `CLOCK_UPTIME_RAW`         | never               |
| [`CoarseMonotonicClock`][t2t::clock::CoarseMonotonicClock] | `Uptime`       | `CLOCK_MONOTONIC_COARSE`   | `CLOCK_UPTIME_RAW_APPROX`  | never               |
| [`RawMonotonicClock`][t2t::clock::RawMonotonicClock]       | `RawUptime`    | `CLOCK_MONOTONIC_RAW`      | `CLOCK_UPTIME_RAW`         | never               |
| [`BootClock`][t2t::clock::BootClock]                       | `BootUptime`   | `CLOCK_BOOTTIME`           | `CLOCK_MONOTONIC`          | never               |
| [`ProcessCpuClock`][t2t::clock::ProcessCpuClock]           | `Timedelta`    | `CLOCK_PROCESS_CPUTIME_ID` | `CLOCK_PROCESS_CPUTIME_ID` | never               |
| [`ThreadCpuClock`][t2t::clock::ThreadCpuClock]             | `Timedelta`    | `CLOCK_THREAD_CPUTIME_ID`  | `CLOCK_THREAD_CPUTIME_ID`  | never               |
| [`Counter`][t2t::clock::Counter]                           | `Tickstamp`    | `cntvct_el0` or `rdtsc`    | `cntvct_el0` or `rdtsc`    | never               |

The operating system's clocks read `clock_gettime` with the id the table gives,
and need the `std` feature, on 64-bit Linux or macOS; `TaiClock` is Linux's
alone. Each id is one its system has, so a reading never fails. The counter
reads `cntvct_el0` on `aarch64` and `rdtsc` on `x86_64`, on any system, with or
without `std`; [The CPU Counter](the-cpu-counter.md) has the rest of it.

Each clock is a unit struct, so `SystemClock.now()` reads one. A reference to a
clock is a clock too, so code takes `&C` where `C: Clock`.

## The Wall Clock

`SystemClock` is the clock whose readings mean something off this machine, so it
stamps a capture another machine will compare. A time service disciplines it,
and may step it backwards across a correction: a span that must never run
backwards is measured on `MonotonicClock` or the counter instead.

`CoarseSystemClock` hands back the value the kernel's timer last wrote, and
reads no counter: the same moment, to the timer's period, a few milliseconds. It
answers whether a heartbeat or an expiry is due; it never stamps a capture.
macOS has no coarse wall clock, so there it reads `CLOCK_REALTIME`.

```text
{{#include ../../crates/t2t/tests/book/os/choosing_a_clock.rs:stamp}}
```

## TAI

`TaiClock` reads International Atomic Time: the wall clock plus the kernel's TAI
offset, which a time service such as `chrony` or `ptp4l` sets to the leap
seconds since 1972. Until one does, the offset is zero, and the clock reads the
wall clock's time. A leap second does not step it, but it steps with the wall
clock when the wall clock is set, and when a time service sets the offset. Linux
alone has the clock.

```text
{{#include ../../crates/t2t/tests/book/os/choosing_a_clock.rs:tai}}
```

## The Monotonic Clocks

`MonotonicClock` never steps, and every process on the machine reads the same
clock, so a deadline is `MonotonicClock.now()` plus a timeout. It stops while
the machine sleeps. Linux lets a time service slew its rate; macOS does not.
`CoarseMonotonicClock` reads the same timeline to the timer's period, for a
deadline that is far off, or polled often; both read an `Uptime`, so their
readings compare.

```text
{{#include ../../crates/t2t/tests/book/os/choosing_a_clock.rs:deadline}}
```

`RawMonotonicClock` runs at the hardware's own rate, which no time service
adjusts: the clock to measure a span free of a time service's corrections, and
the one t2t measures an `x86_64` counter's rate against. It stops while the
machine sleeps too. Its readings are a `RawUptime`, so they never mix with the
monotonic clock's. On macOS it reads the same clock as `MonotonicClock`, which
no time service slews there.

`BootClock` counts the time the machine was suspended, which `MonotonicClock`
does not: the clock for a lease or a timeout that must run down across a
suspension.

```text
{{#include ../../crates/t2t/tests/book/os/choosing_a_clock.rs:raw-and-boot}}
```

## CPU Time

`ProcessCpuClock` reads the CPU time the process has used, on every thread, and
`ThreadCpuClock` the time the calling thread has used, so two threads' readings
count two different times. Each reads a `Timedelta`, the CPU time used so far,
and each takes a system call, not the fast path the clocks above take.

```text
{{#include ../../crates/t2t/tests/book/os/choosing_a_clock.rs:cpu}}
```

## What a Reading Costs

One reading of each, on an AWS Graviton4 under Linux 6.12, pinned to an isolated
core, the median of each of two runs:

| Clock                                                                         | One reading   |
| ----------------------------------------------------------------------------- | ------------- |
| `CoarseSystemClock`, `CoarseMonotonicClock`                                   | 9.8 to 9.9 ns |
| `Counter`                                                                     | 11.8 ns       |
| `SystemClock`, `TaiClock`, `MonotonicClock`, `RawMonotonicClock`, `BootClock` | 33.2 ns       |
| `ThreadCpuClock`                                                              | 269 to 273 ns |
| `ProcessCpuClock`                                                             | 388 to 394 ns |

The run is under
`crates/t2t-clock/benches/results/2026-10-03T10-21Z-8aeae1f-clock-reads/`;
[Performance](performance.md) has the rest, and how to measure on your own
machine. The coarse clocks cost least because they read no counter, and for the
same reason are right only to the timer's period.

## Which to Read

- To stamp a capture, read `SystemClock`: its reading names a moment another
  machine names too.
- To wait, read `MonotonicClock`: it never steps, so a deadline holds.
- To measure, read the [`Counter`][t2t::clock::Counter]: one instruction, and
  one counter for every core and process on the machine.
- To poll, read `CoarseSystemClock` or `CoarseMonotonicClock`: either costs
  least, and is right to the timer's period.
- To run a test or a replay, read a [`ManualClock`][t2t::clock::ManualClock], or
  an [`AtomicManualClock`][t2t::clock::AtomicManualClock] across threads, as
  [Testing with Manual Clocks](testing-with-manual-clocks.md) shows.
