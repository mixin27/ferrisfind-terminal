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

## ADR-008 — Phase 1 fixture shell and terminal lifecycle
**Date:** 2026-10-16

**Status:** Accepted

### Context
Phase 1 adds a navigable shell while actual search and storage remain later gates.
The search field must be useful for keyboard testing without implying backend
results. Terminal teardown must work on failures as well as normal quit paths.

### Decision
Use the resolved, locally inspected Ratatui 0.30.2 and Crossterm 0.29.0 APIs.
Use unicode-segmentation for grapheme editing and unicode-width for cursor columns.
Enable Ratatui's `unstable-rendered-line-info` feature solely to bound inspector
scrolling using the same wrapping algorithm as its renderer; Cargo.lock fixes the
resolved version and render tests cover this API. Review this feature on upgrades.

Show 24 fixed fictional workshop records, with stable fixture IDs, no scores,
one labeled source and no simulated filtering. Enter in search displays an explicit
"not connected" notice and retains the fixtures. This supersedes ADR-007's temporary
status-printing launch behavior. CLI data-directory resolution remains, with no writes.

Use an app-owned reducer and event loop, presentation-only TUI modules and a small
domain fixture shape. Search text is bounded at 256 Unicode scalars; cursor movement
and deletion operate on graphemes. Render wide panels at 120+ columns, an inspector
overlay at 80–119, focused source/inspector views at 60–79, and a resize message
below 60 columns or 16 rows. NO_COLOR/TERM=dumb selects monochrome; indexed colors
are the fallback when truecolor is not advertised. Focus never relies solely on color.

An RAII guard tracks setup attempts before fallible terminal calls and attempts
all cleanup steps even if one fails. Explicit cleanup errors propagate; Drop retries
failed cleanup. Install a chained panic hook once at the application boundary to
restore before printing panic diagnostics. No logs go into the active TUI.

On Unix, signal-hook handlers for SIGINT/SIGTERM/SIGHUP only set an Arc<AtomicBool>;
the event loop checks it through bounded 100ms event polling. This is the sole shared
flag and is required for orderly external interruptions. No worker threads, shared
app state, async runtime or database are introduced. Idle polling does not redraw.

### Consequences
Interactive startup requires stdin/stdout TTYs. CLI help/version remain usable
in pipes. Terminal controls can be failure-tested independently of TTYs; PTY tests
and an opt-in error/panic test validate real teardown separately. SIGKILL, process
abort, and disconnected terminal output cannot be recovered by a guard. All data
remains fictional and ephemeral until a later explicitly requested milestone.

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
