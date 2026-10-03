# Testing with Manual Clocks

Code that takes its clock as a parameter, generic over
[`Clock`][t2t::clock::Clock], reads `SystemClock` in production and a manual
clock in a test. A manual clock reads whatever point it was last set or moved
to, so a test decides what time it is.

```rs
{{#include ../../crates/t2t/tests/book/testing_with_manual_clocks.rs:generic}}
```

## ManualClock

[`ManualClock<P>`][t2t::clock::ManualClock] holds a point of any timeline, a
`Timestamp` unless another is named. `set` moves it to a point, earlier or
later, and `advance` moves it by a span, back for a negative one, saturating as
the point's own addition does. Between moves it stands still, and reading it
costs a load.

```rs
{{#include ../../crates/t2t/tests/book/testing_with_manual_clocks.rs:manual}}
```

A manual clock is not `Clone`: a copy would fork time, the copy and the original
each moving alone. It is shared by reference, and the code under test takes
`&C`.

## Replays

A replay drives the clock from what a capture recorded: the clock is set to each
stamp in turn, and the code under test reads it as it would read the wall clock.

```rs
{{#include ../../crates/t2t/tests/book/testing_with_manual_clocks.rs:replay}}
```

## Threads

A `ManualClock` keeps its point in a `Cell`, so it moves to another thread but
two threads never share one: the compiler refuses it.

```rs
{{#include ../../crates/t2t-clock/tests/compile_fail/a_manual_clock_is_not_shared_across_threads.rs}}
```

```text
{{#include ../../crates/t2t-clock/tests/compile_fail/a_manual_clock_is_not_shared_across_threads.stderr:1:9}}
```

[`AtomicManualClock<P>`][t2t::clock::AtomicManualClock] is the one to share,
across scoped threads by reference or across any by `Arc`. It keeps its point in
an `AtomicI64`, and a thread that reads a point another thread set or moved the
clock to also sees what that thread wrote before it moved the clock. Two threads
that advance it at once both count. It builds where the target has 64-bit
atomics.

```rs
{{#include ../../crates/t2t/tests/book/testing_with_manual_clocks.rs:threads}}
```

Loom models check those promises, trying every interleaving of two threads that
the memory model allows;
[Platforms and Features](platforms-and-features.md#what-ci-checks) says where
they run.

## A Clock of Your Own

A manual clock holds a point, so a clock of CPU time, which reads a span, has no
manual twin. `Clock` is one associated type and one method, so a test writes the
clock it needs:

```rs
{{#include ../../crates/t2t/tests/book/testing_with_manual_clocks.rs:custom}}
```
