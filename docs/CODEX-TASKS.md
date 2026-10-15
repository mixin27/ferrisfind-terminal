# Codex task playbook

Use one prompt per Codex session/change set. Do not ask an agent to implement every phase in one go. For all prompts, add: **"Read AGENTS.md and relevant docs before touching code; run verification commands; summarize outcomes and unresolved gaps."**

## Prompt 0 — Bootstrap

> Implement **Phase 0 only** from `docs/ROADMAP.md`. Set up the Rust binary, CLI flags and clean repository structure. Follow `AGENTS.md`. Do not implement TUI, SQLite or later phases. Add README instructions and relevant tests. Verify cargo fmt/clippy/test/check, and list the exact files changed.

## Prompt 1 — TUI shell

> Implement **Phase 1 only**. Build the modern cyberpunk Ratatui shell from `docs/UI-UX.md` with 20–50 clearly fictional in-memory records. Implement focus-aware key handling, inspector, help, responsive layout, and terminal-restoring guard. Write deterministic reducer and layout tests. No SQLite yet. Verify and report manual TTY checks separately.

## Prompt 2 — SQLite persistence

> Implement **Phase 2 only**. Add `rusqlite` with working FTS5 capability, versioned migration including FTS triggers, repository methods, `init`/`seed` CLI commands and temporary-database integration tests. Follow `docs/DATA-SEARCH.md`. No networking or async. Verify insert/update/delete/index consistency.

## Prompt 3 — Live search

> Implement **Phase 3 only**. Replace in-memory results with actual SQLite FTS5 search using a safe query builder, bound parameters, BM25 ranking, snippets, filters, pagination and record inspector. Implement blank-query logic and Unicode/injection-like tests. Never show fake relevance percentages. Keep UI state consistent.

## Prompt 4 — CSV/JSON imports

> Implement **Phase 4 only**. Build validated and bounded JSON/CSV import CLI commands with transactional writes, deterministic mapping, error reporting, source tracking and tests for malformed inputs and rollback. Keep import fully offline and do not log imported content unnecessarily.

## Prompt 5 — Worker and polish

> Implement **Phase 5 only**. Move DB interactions off the Ratatui event loop into a dedicated worker thread, with typed commands/events and monotonically increasing request IDs. Handle stale results, terminal lifecycle, input responsiveness and optional progress. Avoid gratuitous Tokio or shared-connection mutexes. Add event/state tests.

## Prompt 6 — Release readiness

> Implement **Phase 6 only**. Add docs, cross-platform installation instructions, CI checks, 10k synthetic benchmark harness, TUI manual checklist and regression tests. Measure and report actual results; do not invent performance. List any acceptance criteria still unmet.

## Focused review prompt

> Review this repository against `AGENTS.md`, `docs/PRODUCT.md`, `docs/ARCHITECTURE.md`, `docs/DATA-SEARCH.md`, and `docs/TESTING.md`. Report findings in severity order with precise paths, reproduction/tests and small corrective proposals. Do not modify code unless requested.

## Starting new Codex work

Before a task, paste the corresponding prompt, state the phase, and optionally name target files. After a task, ask for review of its acceptance gate before proceeding. If agent implementation differs from specification, record an ADR before broadening scope.
