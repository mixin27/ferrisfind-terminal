# AGENTS.md — FerrisFind Terminal

This repository implements **FerrisFind Terminal**, a keyboard-first, modern cyberpunk terminal user interface (TUI) for browsing and searching **locally imported, authorized data**. This is a Rust learning project and a usable application. Read `docs/README.md` and the current phase in `docs/ROADMAP.md` before making changes.

## Agent operating contract

1. **Work incrementally.** Implement only the requested milestone or a coherent small slice. Do not implement later phases speculatively. If no milestone is specified, start at Phase 0 of `docs/ROADMAP.md`.
2. **Maintain architectural boundaries:** `tui` is presentation, `app` orchestrates state/actions, `domain` defines types/contracts, `search` provides a backend implementation, `storage` owns database operations, and `importer` handles parsing/validation. No SQL in `tui`.
3. **SQLite FTS5 is the MVP search implementation.** Do not substitute a homegrown search engine, external service, network crawler, PostgreSQL, or a generic ORM. Later integration with FerrisFind Core is planned, not required now.
4. **Prioritize correctness and learning over cleverness.** Prefer simple Rust, explicit ownership, enums, `Result`, typed errors, bounded work and tests. Avoid unnecessary `Arc<Mutex<_>>`, `unsafe`, global mutable state, hidden panics, and gratuitous async.
5. **No external network or unauthorized collection.** Data comes only from explicit user-selected local CSV/JSON files or seeded fictional fixtures. No scanning third-party endpoints, credentials, or private databases. Do not create examples with real personal records.
6. **Dependencies must earn their place.** Prefer Rust std, `ratatui`, `crossterm`, `rusqlite` with a checked FTS5-enabled build, `serde`/`serde_json`, `csv`, `clap`, `thiserror`, and `uuid` only as needed. Check installed API versions instead of assuming exact crate APIs.
7. **No fake performance or fake results.** Sample data must be labeled fictional. UI scores reflect actual backend ranking, not invented percentages. Keep performance budgets as goals until benchmarked.
8. **Build terminal hygiene.** Enter/exit raw mode and alternate screen safely (including errors); restore terminal via guard/RAII; handle resize, small terminal sizes, Ctrl+C and interruptions. Never print ordinary logs into an active alternate screen.
9. **Keep the UI responsive.** Once storage is integrated, a dedicated DB worker owns its SQLite connection. Communicate through channels, use bounded queues as appropriate, and tag requests with monotonically increasing IDs to discard stale search responses. No long DB calls on the rendering thread.
10. **Stable schema and migrations.** Explicit migrations and transactional writes. Maintain FTS5 external-content consistency for insert/update/delete; do not change persisted schemas silently. Index after seeding, and test updates and deletions.
11. **Defensive file handling.** Bound import size/record size, validate Unicode and mappings, reject invalid records with useful errors, avoid path traversal, preserve original data on failed imports, use SQL parameters, and make errors non-fatal to the TUI.
12. **Preserve user changes.** Inspect existing code before edits. Do not overwrite unrelated files, force-reset git, auto-commit, or reformat the entire repository unnecessarily.
13. **Document decisions.** Update `docs/DECISIONS.md` for material architectural choices and `docs/ROADMAP.md` checkboxes only after verified implementation.
14. **Do not claim tests pass unless they ran.** If the environment blocks them, explain what remains unverified.

## Workflow for every task

- Read the relevant documentation and report a short implementation plan.
- Inspect current code and tests; state assumptions where docs leave room for interpretation.
- Implement the smallest vertical slice with appropriate unit/integration tests.
- Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` when a Cargo project exists. During the initial setup, run `cargo check` as well. If tests rely on TTY interaction, separate deterministic state/render tests from manual terminal QA.
- Summarize files changed, commands actually run/results, risks, and the next recommended milestone.

## Important invariants

- One terminal event loop owns `AppState`; workers send typed messages back.
- `q` quits only outside text-editing focus; Esc and Enter depend on focus/screen.
- Cursor position/selection is clamped after every results update.
- Every imported record belongs to a source; record IDs are stable, unique UUID strings.
- FTS5 `rowid` joins a normal `records` table; avoid relying on user-facing text IDs as rowids.
- SQLite `bm25()` returns a *lower-is-better* ranking, not percentages.
- User input is parsed into a safe, narrowly defined search grammar before calling FTS5 `MATCH`; bind the resulting expression as a parameter.
- Query result page size is bounded (default 50, max 200); never pull all records into TUI memory.
- Imports are transactional per chosen import unit; partial success is explicit, never silent.
- Out-of-order query responses must not replace newer results.

## Documentation map

- `docs/README.md`: entry point, scope and project relationships
- `docs/PRODUCT.md`: functional requirements, user stories, keybindings, acceptance criteria
- `docs/UI-UX.md`: layout, palette, focus, animations, small-screen behavior
- `docs/ARCHITECTURE.md`: dependency direction, modules, state/event flows and interfaces
- `docs/DATA-SEARCH.md`: SQLite schema, FTS5 semantics, algorithms and importing
- `docs/ROADMAP.md`: milestones, gates and agent task order
- `docs/TESTING.md`: testing matrix, security/correctness and performance measurement
- `docs/DECISIONS.md`: architectural decision record
- `docs/CODEX-TASKS.md`: copy-paste prompts per phase

## Definition of done

A milestone is done only if its acceptance tests pass, relevant docs are updated, critical error paths are covered, and the resulting CLI/TUI still launches and exits without terminal corruption. Do not skip acceptance criteria to satisfy a deadline.
