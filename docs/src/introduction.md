# Introduction

t2t is a set of Rust crates for time: the instants a program reads off its
clocks, the spans between them, and the clocks themselves. It is written for
code where a nanosecond counts, and where reading one clock's value against
another's is a bug to catch before it runs.

Each timeline has its own point: [`Timestamp`][t2t::Timestamp] on the wall
clock, [`Uptime`][t2t::Uptime] on the monotonic clock, [`Tick`][t2t::Tick] on
the CPU's counter, and others. A point minus a point is a span, a point plus a
span is a point, and two timelines' points never mix. Every point and span is
one `i64`, and every operator saturates at the ends of the range, with a
`checked_*` twin.

Every clock implements [`Clock`][t2t::clock::Clock], whose one verb, `now`,
reads its own kind of point. The [`t2t`] crate re-exports everything; `t2t-core`
holds the values and never reaches the operating system, and `t2t-clock` the
clocks.

This book explains how the pieces fit; the API documentation holds each item's
detail.
