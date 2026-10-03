# The CPU Counter

[`Counter`][t2t::clock::Counter] reads the CPU's free-running counter in one
instruction that touches no memory: `cntvct_el0` on `aarch64`, the time-stamp
counter on `x86_64`. A reading is a [`Tickstamp`][t2t::Tickstamp], the
difference of two a [`Tickdelta`][t2t::Tickdelta], and the counter's
[`TickRate`][t2t::TickRate] turns that into a [`Timedelta`][t2t::Timedelta].

## Ticks and Rates

A tick's length is the counter's: Apple silicon's counter runs at 24 MHz, an Arm
core's from Armv8.6 on at 1 GHz, which is `TickRate::GIGAHERTZ`, and an `x86_64`
core's at the rate its time-stamp counter is rated for. A reading's origin is
the hardware's own and means nothing, so only a span converts:
`Tickdelta::to_timedelta` and `Timedelta::to_tickdelta`, each at a rate.

```rs
{{#include ../../crates/t2t/tests/book/the_cpu_counter.rs:rate}}
```

## Discovering the Counter

`Counter::discover` finds the counter's rate, or refuses with a
[`CounterError`][t2t::clock::CounterError] that says why there is none to read
as time:

- On `aarch64` it reads `cntfrq_el0`, the rate the firmware wrote.
- On `x86_64` it refuses a time-stamp counter that is not invariant, by CPUID
  leaf `0x8000_0007`. It takes the rate from CPUID leaf `0x15`, a crystal's
  frequency times a ratio; where that has none, from the hypervisor's leaf
  `0x4000_0010`; and where neither has one, with `std` on 64-bit Linux or macOS,
  it measures the rate over 10 ms against `RawMonotonicClock`.
- On either, a rate outside 1 MHz to 10 GHz is refused.

| Refusal           | When                                                 | Why                                                                                                                                    |
| ----------------- | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `NotInvariant`    | the time-stamp counter's rate follows the core's     | its ticks have no fixed length, so no rate makes them time                                                                             |
| `NoRate`          | the CPU reports no rate, and none could be measured  | nothing says what a tick is worth                                                                                                      |
| `ImplausibleRate` | the rate read or measured is outside 1 MHz to 10 GHz | below a tick a microsecond a counter measures nothing worth measuring, and none runs past 10 GHz, so a rate outside is taken for wrong |

`NoRate` comes only on `x86_64`, where CPUID reports no rate and none could be
measured, as without `std`, or off 64-bit Linux and macOS. Each refusal's
message says which it is: `counter error: a rate of 42 Hz is outside 1 MHz to 10
GHz`.

```rs
{{#include ../../crates/t2t/tests/book/the_cpu_counter.rs:discover}}
```

A measurement takes 10 ms, so a program discovers its counter once, at start-up,
and keeps it: a `Counter` is `Copy`, and holds only its rate.

## Virtual Machines

A hypervisor chooses the CPUID bits its guest sees. One that does not promise an
invariant counter makes `discover` refuse with `NotInvariant`, even where the
host's counter is invariant: the Intel macOS virtual machines t2t's CI runs on
are such guests, and refuse. One that reports the counter's rate in leaf
`0x4000_0010` spares the guest the measurement.

Where the rate is known, from a config or from a discovery made elsewhere,
`Counter::new` takes it and checks nothing: neither that the counter is
invariant, nor that the rate is plausible. The caller vouches for both. Where
`discover` refuses and no rate is known, read `MonotonicClock` instead.

```rs
{{#include ../../crates/t2t/tests/book/the_cpu_counter.rs:known-rate}}
```

## One Counter for the Machine

Every core and every process on a machine reads the same counter, so a
`Tickstamp` one process takes, another may subtract from. t2t-clock's test
[`counter_cross_process`](https://github.com/atomix-labs/t2t/blob/main/crates/t2t-clock/tests/counter_cross_process.rs)
starts a child process, and checks that the reading the child prints lies
between two the parent took around it. A reading crosses as its spelling, `1000
ticks`, or as its count. Since only spans convert, two processes that measured
slightly different rates still agree on every reading; they differ only in what
a span is worth.

The read is not ordered against the instructions around it, which is right for a
stamp, and for timing anything much longer than the read itself. `Clock::now` on
a `Counter` is `#[inline(always)]`, since a call around the read would move it.

## The Conversion

`TickRate::from_hertz` works out, once, a factor $f$ and a shift $s$ for each
direction, so a conversion is a 128-bit multiply and a shift on the count's
magnitude, with its sign put back, and no division. For a ratio of $n$ to $d$,
$10^9$ to the rate for ticks to nanoseconds and the rate to $10^9$ the other
way:

$$
f = \left\lceil \frac{n \cdot 2^{s}}{d} \right\rceil
\qquad
\mathrm{converted}(c) = \left\lfloor \frac{|c| \cdot f}{2^{s}} \right\rfloor
$$

The shift is the largest whose factor still fits 64 bits, so the factor carries
every bit of precision it can. Rounding it up makes an exact multiple of the
ratio convert exactly, and any other count land within one unit of its exact
quotient, which a property test checks against an `i128` division, on counts
drawn from every `i64` and rates from every `u64` above zero. A result past the
range saturates.

```rs
{{#include ../../crates/t2t/tests/book/the_cpu_counter.rs:accuracy}}
```

A conversion takes about 1.2 ns from ticks to nanoseconds and 1.0 ns the other
way, at 24 MHz on a Graviton4: 1,024 of them in 1.206 µs and 1.034 µs, in the
run under
`crates/t2t-core/benches/results/2026-10-02T23-14Z-9776c95-span-divisors/`.
