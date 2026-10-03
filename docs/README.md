# The Book

The book's source, which mdBook builds into `book/`. Every page is a Markdown
file under `src/`, listed in `src/SUMMARY.md`: a page missing from it is never
published, and the checks fail on it.

- `just check-mdbook` lints the book and builds it; `just check` runs its Rust
  listings, as the tests below.
- `mdbook serve` in this directory serves it, rebuilt on every change.

A Rust listing on a page is included by its anchor from a test that cargo runs:
`crates/t2t/tests/book/`, a module a chapter, or `crates/t2t-core/tests/book/`
for a listing that needs a crate the facade does not depend on, such as
`serde_json`. `mdbook test` passes no crate of the workspace to rustdoc, so a
listing that uses one would not compile there. Each is fenced `rs`, which the
book's highlighter colours as Rust and rustdoc leaves alone.

The book's own look is in `theme/`: the page width, collapsible definitions, and
a `$` prompt on `console` blocks.
