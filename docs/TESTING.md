# Testing, quality and security plan

## Automated baseline

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check
```

Use `tempfile` for isolated database directories; tests must not write into real user data dir. Unit-test reducers and import validators without terminal I/O. Integration-test SQLite on temp databases. Mark TTY-only manual tests separately.

## Phase 0 verification — 2026-10-15

Environment: macOS aarch64, stable rustc 1.99.0, Cargo 1.99.0, edition 2024.
Dependencies were resolved from the local Cargo cache without network access.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo check` (also `cargo check --offline`): passed.
- `cargo test`: 8 unit tests and 5 CLI integration tests passed.
- Check, clippy and tests also passed with `--locked --offline` in a fresh
  temporary target directory, verifying a clean build without deleting existing artifacts.
- `cargo run -- --help`, `cargo run -- --version`, and
  `cargo run -- --data-dir './local data'`: passed.
- Separate PTY smoke check of default launch, help and invalid arguments:
  expected exit codes 0/0/2, identical terminal attributes before/after, no ANSI
  escape sequences. This is a Phase 0 check, not manual TUI QA.

Coverage includes native/non-UTF-8 paths, Unicode paths, platform defaults,
override precedence, missing configuration, empty/missing/repeated arguments,
unsupported future commands, output I/O failure and escaped diagnostics.
Integration tests remove directory environment variables only in child processes
and assert that startup/help/version do not create the selected data directory.
No test writes to the real application-data directory.

The first test run exposed an incorrect expectation of unescaped Myanmar combining
marks in path output; the assertion now checks the documented escaped representation.
The complete verification set passed after that correction.

The CI workflow repeats the baseline and help check on Linux, macOS and Windows.
Hosted CI, Windows defaults and Linux XDG behavior have not been executed locally;
platform-specific tests will run on those CI hosts. TUI manual QA belongs to Phase 1.

## Phase 1 verification — 2026-10-16

Environment: macOS aarch64, stable rustc/Cargo 1.99.0. Ratatui 0.30.2,
Crossterm 0.29.0. Build dependencies were fetched from crates.io; runtime is offline.

### Deterministic automated checks

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo check`: passed.
- `cargo test`: 24 unit tests and 5 CLI tests passed; one TTY-only test is ignored
  by default and was run separately below.
- `cargo run -- --help`: passed without entering terminal mode.
- `git diff --check`: passed.

Reducer tests cover focus cycling, modal return, selection clamping after shorter
and empty record sets, grapheme editing, paste bounds, and inspector scroll bounds.
Keymap tests distinguish literal q/j/k/slash in search from navigation, ignore
release events and check Ctrl+C from every focus. Layout and TestBackend render
tests cover zero-sized areas, all breakpoints, minimum-height fallback, empty
records, last-record visibility, narrow focused views and full help at 60×16.
Guard tests inject partial setup failures, cleanup write failures, early errors
and panic unwinding. CLI tests cover help/version, invalid arguments, missing
configuration, non-TTY rejection and no directory creation.

### Interactive PTY inspection (separate from deterministic tests)

Launched `cargo run --offline -- --data-dir ./phase1-demo` in an interactive
130×30 PTY and sent keys manually through the terminal tool:

- `/`, then `q`, Myanmar text and an emoji: text remained in the search field,
  cursor moved, and q did not quit.
- Enter: returned to records with the explicit search-not-connected notice.
- Enter again, then j/j: inspector focused and scrolled.
- F1 twice: help opened and returned to the inspector.
- q: exited with code 0; before/after `stty -g` matched and output included cursor,
  bracketed-paste and alternate-screen restoration.

This was agent-driven PTY inspection, not a human visual check in a desktop terminal.

### Reproducible PTY and error checks

`cargo build --offline` and `python3 scripts/pty_smoke.py` passed all six scenarios:

1. Search-focus q, bracketed Unicode paste, inspector, resize through 100×24,
   70×20, 60×16, 40×10 and back to 130×30; help does not leak q; normal q exit.
2. Ctrl+C while editing.
3. External SIGINT.
4. External SIGTERM.
5. External SIGHUP.
6. `cargo test --offline real_tty_error_and_panic_cleanup -- --ignored --nocapture
   --test-threads=1` inside a PTY: injected error and caught panic both restore;
   panic diagnostics follow the leave-screen sequence.

The first five scenarios verify raw-mode entry, no idle redraw, exit 0, exact
termios restoration and cursor/paste/screen cleanup. In the error/panic harness,
macOS can set the transient `PENDIN` (retype pending input) state bit on restoration;
the comparison excludes only that kernel state bit and verifies all other terminal
settings. The expected simulated panic is caught; that test passes.

The PTY harness answers cursor-position requests because a bare PTY has no terminal
emulator to answer them. Initial harness attempts without this response could not
complete initialization; the committed harness includes it and passed.

### Remaining verification limits

Linux/Windows runtime behavior and hosted CI have not been executed in this session.
The Python PTY suite is Unix-only; Windows still runs deterministic Cargo tests in
the existing CI matrix. Desktop terminal font rendering and native resize behavior
need human/platform QA. The Ratatui rendered-line-info feature is currently marked
unstable and must be reviewed during dependency upgrades. Database, import, actual
search and worker checks below belong to later milestones.

## Required test matrix

| Domain | Cases |
|---|---|
| Navigation | Type `q` in search; `q` in results quits; Enter/Esc routing; Tab focus; F1 help |
| Terminal | Small terminals, resize while selected, Ctrl+C, early error cleanup, Unicode clipping |
| State | Empty result list, selection at end when results shrink, stale request ID ignored |
| Import JSON | Valid array, missing title, wrong types, duplicate IDs, huge content, invalid UTF-8 |
| Import CSV | Quoted commas, embedded newlines, BOM behavior, missing header, extra metadata columns |
| Transactions | Failure mid-import rolls back; counts reflect committed records |
| FTS | Insert searchable, update removes old term, delete removes term, source delete does not leave hits |
| Search | Unicode text, punctuation, quotes, operator-looking text, zero tokens, long query, blank query |
| Ranking | Known more-relevant record sorts correctly; scores never presented as percentages |
| Filtering | Source/category combination, no matching category, stable page ordering |
| Storage | Migration repeatable, database reopen, no unexpected database writes on `--help` |
| Safety | SQL injection-like string doesn't execute SQL; ANSI/control text sanitized in UI |

## Deterministic fixture

Create 20–50 fictional records with overlapping terms, distinct categories, Unicode and Myanmar text samples, long descriptions, quote marks, duplicate titles, and deliberately malformed import fixtures. Label all data **fictional**. Add a seeded generator for 1k/10k/100k synthetic records; fix RNG seed.

## Benchmark protocol

Benchmarks are engineering measurements, not promises. Record OS, CPU, RAM, database size, dataset generation seed, number of records, warm/cold cache, and runs. Measure p50/p95 latency for common/rare queries, imports per second, peak memory, UI input latency during import. A **target** for typical local 10k-record indexed queries is <100 ms on a development laptop; revise after actual measurements and record evidence. Never hardcode these numbers as UI status.

## Terminal manual QA

1. Enter app from a standard shell; verify alternate screen and raw mode.
2. Search, navigate, inspect, resize repeatedly, open/close help.
3. Type letters corresponding to shortcuts while search is focused.
4. Exit via `q`, Ctrl+C and recoverable error; ensure cursor visibility, echo and shell are restored.
5. Test reduced terminal width and height; inspect long Unicode strings/emoji.
6. Start heavy import and verify input responsiveness, progress and safe cancellation behavior.

## Security boundaries

Never make unsolicited network connections, crawl websites or access external databases. Accept only explicitly selected local sources. Imported text is untrusted and must not be interpreted as terminal control sequences or SQL. Protect against resource-exhaustion via limits, timeout/cancellation strategy, bounded results and bounded work queue. Follow principle of least surprise regarding files in user data dir.
