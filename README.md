# FerrisFind Terminal

A planned offline-first, keyboard-driven, modern cyberpunk database search terminal written in Rust.

**Status:** Phase 1 TUI shell with 24 clearly fictional in-memory records.
Browse records, edit a search field, inspect details and open keyboard help.
Search submission explains that search is not connected and leaves the fixtures
unchanged. SQLite storage, actual search, seed/import and doctor commands remain future work.

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
Runtime is offline. Interactive launch requires a terminal on stdin and stdout;
help and version work without one. Cargo may need network access to download
build dependencies on first build.

## Keyboard and layout

| Key | Action |
|---|---|
| `/` | Focus the search field |
| Tab / Shift+Tab | Cycle search, sources, records and inspector |
| Up/Down or j/k | Browse records or scroll the focused inspector |
| Home / End | First/last record or inspector line; beginning/end when editing |
| Enter | Submit field, open inspector, or close inspector/help |
| Esc | Return to records or close help |
| Left/Right, Backspace/Delete | Edit by grapheme while searching |
| Ctrl+L / Ctrl+R | Clear field / refresh the in-memory fixture count |
| F1 | Open/close help and return to the prior focus |
| q | Quit outside search and help; typed literally in search |
| Ctrl+C | Quit from any focus |

Search input is limited to 256 Unicode characters; bracketed paste is supported.
The source panel describes the single fictional workshop source. No filters or
ranked results are simulated. All changes are discarded on exit.

At 120+ columns the source, record list and inspector share the screen. At 80–119
columns the inspector overlays the workspace; at 60–79 columns sources and the
inspector are separate focused views. Below 60 columns or 16 rows, a resize message
replaces the workspace; Ctrl+C still exits, or press Esc then q.

The palette uses RGB when `COLORTERM=truecolor` or `24bit`, basic terminal colors
otherwise, and monochrome for `NO_COLOR` or `TERM=dumb`. Focus also uses thick
borders and `>` markers. Unicode rendering depends on the terminal/font.

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
Phase 1 resolves the path for later storage integration but does not display,
create, validate accessibility of, or write to that directory.
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
defaults, focus routing, selection bounds, Unicode editing, responsive rendering,
terminal guard error paths and absence of startup writes; no real user data is used.

TTY checks run separately from the deterministic tests:

```sh
cargo build
python3 scripts/pty_smoke.py  # macOS/Linux; Python standard library only
```

This checks resize, Unicode paste, help isolation, q/Ctrl+C, Unix termination
signals, idle behavior and terminal restoration. It also runs the ignored
`real_tty_error_and_panic_cleanup` test inside a PTY; its simulated panic is
expected and caught. For direct interactive execution of that one test:

```sh
cargo test real_tty_error_and_panic_cleanup -- --ignored --nocapture --test-threads=1
```

See [`docs/TESTING.md`](docs/TESTING.md) for verified results and manual QA limits.

Exit codes: `0` for successful launch/help/version, `2` for invalid arguments,
`1` for configuration or terminal errors. Clap prints usage diagnostics to stderr;
application failures use one escaped key/value line such as
`level=error event=configuration_failed message="..."`.
No log files or telemetry are created. The guard restores raw mode, alternate
screen, cursor visibility and paste mode before application errors are reported.
A chained panic hook restores the terminal before panic diagnostics. On Unix,
SIGINT/SIGTERM/SIGHUP request orderly shutdown; Ctrl+C is handled as a key event
on all platforms. SIGKILL and terminal disconnection cannot guarantee screen restoration.

## Start with Codex

1. Read [`AGENTS.md`](AGENTS.md).
2. Review the documentation index at [`docs/README.md`](docs/README.md).
3. Use the next unfinished phase prompt from [`docs/CODEX-TASKS.md`](docs/CODEX-TASKS.md).
4. Proceed through [`docs/ROADMAP.md`](docs/ROADMAP.md) only after each acceptance gate passes.

Next milestone: Phase 2, persistent SQLite storage with migrations and FTS5 verification.
