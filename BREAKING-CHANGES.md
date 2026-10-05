# Breaking Changes

A migration note for every breaking change, newest release first: what changed,
why, and what a user does about it.

[CHANGELOG.md](CHANGELOG.md) lists every change; this lists only those a user
must act on.

## Summary

- [v0.2.0](#v020)
  - [The `zerocopy` feature is `zerocopy-08`](#the-zerocopy-feature-is-zerocopy-08)

## V0.2.0

### The `zerocopy` Feature Is `zerocopy-08`

**What changed.** The feature that derives the zerocopy traits is `zerocopy-08`,
in `t2t` and `t2t-core` alike. A feature that brings a crate before 1.0 names
its version, as `chrono-04`, `jiff-02` and `time-03` do, so zerocopy's next 0.y
can come beside it without another breaking release. What the feature adds is
unchanged.

**What to do.** Rename the feature where you turn it on, on `t2t` or `t2t-core`:

```toml
t2t = { version = "0.2.0", features = ["zerocopy-08"] }
```

A feature of your own that forwards to it forwards to the new name: `zerocopy =
["t2t/zerocopy"]` becomes `zerocopy = ["t2t/zerocopy-08"]`.
