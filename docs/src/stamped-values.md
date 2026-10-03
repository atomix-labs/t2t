# Stamped Values

[`Timed<T, S>`][t2t::Timed] holds a value `T` and the stamp `S` its writer
captured it with, a `Timestamp` unless a stamp of another timeline is named. It
says how old the value is at any later point, and reaches the value through
`Deref`, so a stamped sample's fields read as the sample's own.

```rs
{{#include ../../crates/t2t/tests/book/stamped_values.rs:timed}}
```

`elapsed` is the span from the stamp to the point it is given, on the stamp's
own timeline, and negative for a stamp still ahead of it.

## The Writer Stamps

The code that captures a value stamps it, and whatever reads it keeps that
stamp: by the time a reader holds the value, the moment it was captured has
gone, and a clock read on the reading side would time the reader. `map` makes a
new value under the same stamp, and `map_stamp` a new stamp over the same value.

```rs
{{#include ../../crates/t2t/tests/book/stamped_values.rs:writer}}
```

`as_ref` and `as_mut` borrow the value under the same stamp; `into_inner` gives
the value, `into_parts` the stamp and the value, and a `Timed` converts to and
from the pair `(S, T)`.

## Order

The stamp is the first field, so the order a `Timed` derives is chronological,
and the value decides only between equal stamps.

```rs
{{#include ../../crates/t2t/tests/book/stamped_values.rs:order}}
```

## Layout

A `Timed` is `#[repr(C)]`: the stamp at offset zero, then the value. A
`Timed<u64>` is 16 bytes, with no padding, on any timeline, since every stamp is
an `i64`. With `zerocopy`, a `Timed` is read from bytes, as
[Serialization](serialization.md#bytes) shows.

```rs
{{#include ../../crates/t2t/tests/book/stamped_values.rs:layout}}
```
