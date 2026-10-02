# Getting Started

Add the crate, with the system clocks:

```sh
cargo add t2t --git https://github.com/atomix-labs/t2t --features std
```

Two examples walk through the rest. The first prints the values: instants and
spans, the calendar, a stamped value, and a counter's rate.

```sh
cargo run -p t2t --example time-tour
```

The second prints every clock's reading, and what one costs on the machine it
runs on.

```sh
cargo run --release -p t2t --features std --example clock-tour
```

From there, [`clock`][t2t::clock] lists every clock, and the crate's page,
[`t2t`], says which to choose for what.
