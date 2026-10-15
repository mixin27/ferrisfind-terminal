# Architecture decision log

Use dated entries with **Context**, **Decision**, **Consequences**, **Status**. An agent may propose a change but should not silently override accepted choices.

## ADR-001 — Local-first MVP
**Status:** Accepted.  
**Decision:** Search explicit locally imported CSV/JSON and fictional seed data. No remote scraping, external database connection or auth in MVP.  
**Why:** Keeps setup simple, privacy-preserving, testable and aligned to Rust learning.

## ADR-002 — Ratatui + Crossterm
**Status:** Accepted.  
**Decision:** Use Ratatui for rendering and Crossterm for terminal events, raw mode and alternate screen.  
**Consequence:** Terminal restoration and display-width handling are central correctness requirements.

## ADR-003 — SQLite + FTS5 first
**Status:** Accepted.  
**Decision:** SQLite record store and built-in FTS5 indexing/BM25 for MVP; custom search index remains in FerrisFind Core.  
**Consequence:** Must enforce trigger/index consistency and safely generate FTS expressions.

## ADR-004 — Single package before workspace
**Status:** Accepted.  
**Decision:** Begin with a modular single Cargo crate. Split into workspace only when reusable Core/CLI crates demonstrably need it.  
**Consequence:** No premature crate graph.

## ADR-005 — Worker owns SQLite connection
**Status:** Accepted for storage integration.  
**Decision:** Dedicated worker thread executes DB commands; TUI keeps AppState on event loop thread. Search request IDs reject stale responses.  
**Consequence:** Explicit channels and shutdown/error behavior; avoids shared connection mutexes.

## ADR-006 — Cinematic, not deceptive
**Status:** Accepted.  
**Decision:** Cyberpunk palette and restrained motion; record counts, scores and results always derived from actual storage or labeled fictional fixtures.  
**Consequence:** No artificially delayed searches or fabricated relevance percentages.

## ADR-007 — Phase 0 CLI and diagnostics
**Date:** 2026-10-15

**Status:** Accepted

### Context
Phase 0 needs a runnable CLI, configuration, errors and CI without implementing
the TUI or storage. The existing Rust 2024 package is supported by installed stable
Rust 1.99.0. Normal launch behavior and the diagnostic format were unspecified.

### Decision
Keep the single `ferrisfind-terminal` package and name its binary `ferrisfind`.
Use the locally inspected Clap 4.6.7 builder API with only std/help/usage/error-context
features, and commit Cargo.lock. Use std for configuration and typed application
errors; no TUI, database, async or logging-framework dependencies yet.

Resolve native `PathBuf` values from `--data-dir` first, then platform defaults:
macOS HOME/Library/Application Support, Windows LOCALAPPDATA, and otherwise
XDG_DATA_HOME or HOME/.local/share, each with a `ferrisfind` child directory.
Environment bases must be absolute; unusable XDG values fall back to HOME.
No usable default produces an actionable error. Explicit relative paths are allowed.
Resolution does not create directories or check permissions. Help/version bypass it.

Bare launch prints an honest Phase 0 notice and the escaped selected path, then
exits. Application failures are escaped single-line key/value stderr records with
`level`, stable `event`, and `message` fields. Clap retains its readable usage errors.
Exit codes are 0 (success), 1 (application failure), 2 (usage error). No logs contain
imported content; there are no log files or telemetry. When the TUI arrives, stderr
diagnostics must wait until terminal restoration, or use a dedicated file sink.

### Consequences
The CLI is usable without a TTY or filesystem writes, and tests can inject environment
lookups without mutating global state. Later phases add actual operations and terminal
lifecycle handling at their gates. CI checks stable Rust on Linux, macOS and Windows;
only results actually run locally or in CI may be reported as verified.

## ADR template

```markdown
## ADR-NNN — Title
**Date:** YYYY-MM-DD  
**Status:** Proposed | Accepted | Superseded

### Context
...
### Decision
...
### Consequences
...
```
