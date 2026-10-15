# FerrisFind Terminal — Documentation index

**Status:** Phase 0 CLI foundation implemented; later phases remain specifications

**Version:** 1.0  
**Primary language:** Rust  
**Product:** Local-first, cyberpunk-themed database search TUI

## Start here

1. Read [`PRODUCT.md`](PRODUCT.md) for the exact MVP.
2. Read [`UI-UX.md`](UI-UX.md) for the interactive screen design.
3. Read [`ARCHITECTURE.md`](ARCHITECTURE.md) for module ownership and event flow.
4. Read [`DATA-SEARCH.md`](DATA-SEARCH.md) for database schema, indexing and safe query rules.
5. Execute [`ROADMAP.md`](ROADMAP.md) one gated milestone at a time using [`CODEX-TASKS.md`](CODEX-TASKS.md).
6. Apply [`TESTING.md`](TESTING.md) and record deviations in [`DECISIONS.md`](DECISIONS.md).

## Relationship with FerrisFind Core

**FerrisFind Terminal** is a standalone TUI, initially backed by SQLite FTS5. **FerrisFind Core** (separate planned project) teaches file scanning, custom inverted indexes, Boolean/positional search and performance engineering. Do **not** build Core inside the Terminal MVP. A future `SearchBackend` adapter can connect them. See `docs/FERRISFIND-CORE-REFERENCE.md` for compatibility notes; that reference is contextual, not a requirement to implement Core.

## What ships in MVP

Full-screen TUI; responsive layouts; keyboard-first interactions; fictional sample data; SQLite record store; JSON/CSV import commands; full-text search with ranking, snippets, basic filtering and pagination; record inspector; worker-based I/O; error messages; automated tests; installable binary.

## Explicitly excluded

Real-world personal intelligence dossiers, third-party scraping, credentials discovery, remote system access, internet connectors, arbitrary SQL execution, AI agents, PostgreSQL connectivity, cloud sync and custom-built search index. Cosmetic terminals must display real, local results.

## Example commands (proposed API, not yet implemented)

Phase 0 implements `ferrisfind --help`, `ferrisfind --version` and `--data-dir PATH`.
Bare launch reports the configured directory and exits without filesystem writes.
The TUI and subcommands below remain planned. See the root README for current usage.

```sh
ferrisfind
ferrisfind init
ferrisfind seed --count 1000
ferrisfind import --file ./sample-data/records.json --format json
ferrisfind import --file ./sample-data/records.csv --format csv
ferrisfind doctor
ferrisfind --help
```

Store application data in an OS-appropriate application-data directory; support `--data-dir` to override for development and tests. CLI commands must not require a TTY except the interactive launch.
