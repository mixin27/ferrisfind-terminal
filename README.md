# FerrisFind Terminal

A planned offline-first, keyboard-driven, modern cyberpunk database search terminal written in Rust.

**Status:** Phase 0 CLI foundation. Help, version, data-directory selection and
error reporting work. Normal launch prints the selected directory and exits;
the TUI, SQLite storage, search, seed/import and doctor commands are future work.

## Build and run

Install stable Rust with Cargo, rustfmt and Clippy (Rust 2024 edition; verified
locally with Rust 1.99.0). From this repository:

```sh
cargo build --locked
cargo run -- --help
cargo run -- --version
cargo run -- --data-dir "./local data"
cargo run
```

The package is `ferrisfind-terminal`; its executable is `ferrisfind`.
To install it locally, run `cargo install --path . --locked`, then `ferrisfind --help`.
Runtime is offline and does not require a TTY in Phase 0. Cargo may need network
access to download build dependencies on first build.

## Data directory

`--data-dir PATH` overrides the platform default:

| Platform | Default |
|---|---|
| macOS | `$HOME/Library/Application Support/ferrisfind` |
| Linux / other Unix | `$XDG_DATA_HOME/ferrisfind`, or `$HOME/.local/share/ferrisfind` |
| Windows | `%LOCALAPPDATA%\ferrisfind` |

Environment defaults must be absolute paths. Empty or relative `XDG_DATA_HOME`
falls back to `HOME`. If no usable default exists, pass `--data-dir PATH`.
Explicit relative paths use the current working directory. Paths remain native
OS paths (including non-UTF-8 paths on Unix); quote paths containing spaces.
Phase 0 only resolves and displays the path, with control characters escaped.
It does not create, validate accessibility of, or write to that directory.
Help and version do not need directory configuration.

## Verification and diagnostics

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check
cargo test
```

CI runs these checks and the help smoke test on stable Rust for Linux, macOS and
Windows, using the committed `Cargo.lock`. Tests cover CLI behavior, platform
defaults, error paths and absence of startup writes; no real user data is used.

Exit codes: `0` for successful launch/help/version, `2` for invalid arguments,
`1` for configuration or output errors. Clap prints usage diagnostics to stderr;
application failures use one escaped key/value line such as
`level=error event=configuration_failed message="..."`.
No log files or telemetry are created. Future TUI code must restore the terminal
before emitting stderr diagnostics, or route logs to a dedicated file while active.

## Start with Codex

1. Read [`AGENTS.md`](AGENTS.md).
2. Review the documentation index at [`docs/README.md`](docs/README.md).
3. Use the next unfinished phase prompt from [`docs/CODEX-TASKS.md`](docs/CODEX-TASKS.md).
4. Proceed through [`docs/ROADMAP.md`](docs/ROADMAP.md) only after each acceptance gate passes.

Next milestone: Phase 1, the terminal shell with explicitly fictional in-memory records.
