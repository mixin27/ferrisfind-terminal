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
