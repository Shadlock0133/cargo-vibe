# Installation

```shell
cargo install cargo-vibe
```

# Running

```shell
cargo vibe build
```

# Configuration

## Vibe Pattern

By default, `cargo-vibe` will, on success, vibe full strength for 3 seconds.

You can change that by setting `CARGO_VIBE_PATTERN` environment variable. For
example, to set it vibe for 1.5 second on 20% strength, you can do:

```shell
CARGO_VIBE_PATTERN="0.2 1.5s" cargo vibe <cmd>
```

You can also set full patterns of vibes to run, by separating them with slashes
`/`. Here is one example:

```shell
CARGO_VIBE_PATTERN="0.4 1s/0.6 1s/0.8 0.75s/1.0 0.25s"
```

## Server Address

If your server is running at an address or port other than the default (`ws://127.0.0.1:1234`), you can override it using `CARGO_VIBE_ADDR`. For example:

```shell
CARGO_VIBE_ADDR="ws://172.31.160.1:69696"
```
