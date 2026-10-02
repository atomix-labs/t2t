# Working in `t2t`

What an agent needs to work here: what the repository is, how to check a change,
and the rules a change keeps.

<!-- >>> devset: agents >>> -->

## Before You Commit

Run `just check`: CI runs the same checks, and names each that fails. `just fix`
fixes what a formatter or linter can, and `just --list` shows every recipe.

## Before You Finish

A change beyond a line or two is ready when its passes have run and `just check`
passes:

- `/humanize` on what the change says to a reader: its comments, docs, names and
  messages.
- `/review-rust` on a change to Rust code.
- `/review-names` on a change that adds, renames or changes what a public item
  does.
- `/review-security` on a change that reads input from outside the process.

Each runs in a fresh context and reports: `/humanize` edits, then says what it
changed and what it left, and a review says what it found. Fix what a pass
leaves or finds, or say why a finding does not hold.

## Managed Files

Profiles, applied by devset, manage some of the files here. `devset status`
names each, and whether a local change to it is kept or is drift; `devset
explain <file>` says which profile owns what in it. What a profile owns changes
with the profile, on `devset update`. Never edit `.devset/`.

<!-- <<< devset: agents <<< -->

## The Repository

A Cargo workspace of three crates under `crates/`: `t2t-core` holds the values
(points, spans, rates, the calendar, `Timed`), `t2t-clock` the clocks, and the
facade `t2t` re-exports both, with the examples. Each inherits its version,
edition, licence and lints from the root `Cargo.toml`. The book is under
`docs/`. The toolchain is the nightly `rust-toolchain.toml` pins; the tools are
the versions `.config/mise/` pins. `.github/workflows/platforms.yml` is the
repository's own: the tests on arm64 Linux and macOS, the bare-metal builds, and
the feature powerset.

## Rules

- `t2t-core` never reaches the operating system: no `libc`, no `std` beyond its
  `std` feature's conversions.
- `unsafe` lives in `t2t-clock` alone, each block under an
  `#[expect(unsafe_code, reason = "…")]` with a `// SAFETY:` comment that
  discharges what the operation requires.
- Every operator of a point or a span saturates, and has a `checked_*` twin; the
  operators come from `ops.rs`'s macros, so every point and span has the same
  set.
- A new timeline is a point of its own: its type in `t2t-core`, `point!` and,
  for nanoseconds, `nanosecond_units!` in `ops.rs`, its name in `clippy.toml`'s
  `arithmetic-side-effects-allowed`, and its re-export in the facade.
- A new clock is one `system_clock!` in `t2t-clock/src/system.rs`, or a file of
  its own implementing `Clock`, and a row in the clock tables of `t2t-clock`'s
  crate page, the facade's and the README.
- Every name is whole words, never a fragment such as `at`, `by` or `held`.

<!-- >>> devset: cargo-deny >>> -->

## Dependencies

`just check-cargo-deny` holds every dependency to `deny.toml`: its advisories,
its licence, its source, and the bans. A failure names a choice for the
maintainer, between a newer version, another crate, and an exception with its
reason: ask before adding an exception or allowing another licence.

<!-- <<< devset: cargo-deny <<< -->

<!-- >>> devset: git-commits >>> -->

## Commits

A pull request lands squashed, as one commit its title names: the title follows
Conventional Commits, `type(scope): subject`, the subject imperative and lower
case, with no closing period, since it is the line the changelog shows; the
workflow `title` checks it. Each commit on a branch keeps the same rules, which
`just check-git-commits` checks. A breaking change adds `!` after the scope, and
a footer that starts `BREAKING CHANGE:` and says what to do.

<!-- <<< devset: git-commits <<< -->

<!-- >>> devset: mdbook >>> -->

## The Book

`just check-mdbook` lints the book, builds it and runs its examples. Its pages
are Markdown under the `src/` of its directory, each listed in `SUMMARY.md`, and
a preview rebuilds on every save:

```sh
mdbook serve docs
```

<!-- <<< devset: mdbook <<< -->
