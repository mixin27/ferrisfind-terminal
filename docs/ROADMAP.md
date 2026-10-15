# Milestone roadmap — agent execution gates

**Rule:** Work in order unless explicitly asked otherwise. A checkbox can be checked only when behavior is implemented **and** verified. Avoid creating empty future modules.

## Phase 0 — Repository foundation

- [x] Initialize a single Cargo binary package with edition supported by installed stable Rust.
- [x] Establish `src/main.rs`, `.gitignore`, `README.md`, `docs/` and `AGENTS.md`.
- [x] Add CLI help/version and `--data-dir` override; `ferrisfind --help` runs.
- [x] Add error type, structured logging strategy and CI checks.
- [x] Run fmt, clippy, check, tests on clean repo.

**Gate:** `cargo run -- --help` works, test suite passes. No TUI architecture prematurely added.

Verified 2026-10-15 on macOS aarch64 with stable Rust 1.99.0: fmt, clippy,
check, 13 tests and CLI smoke checks pass. PTY launch/help/usage-error checks
leave terminal attributes unchanged. Check, clippy and tests also pass with
`--locked --offline` in a fresh temporary target directory. Existing user staging
is preserved; the working tree has not been reset or committed.
CI is configured for Linux/macOS/Windows; hosted CI and non-macOS execution remain
unverified. See `TESTING.md` for the Phase 0 record.

## Phase 1 — TUI shell with fictional in-memory records

- [ ] Enter/restore raw mode and alternate screen via guard.
- [ ] Implement responsive wide/medium/narrow screen layouts.
- [ ] Implement header, search bar, result list, inspector, footer, help overlay.
- [ ] Implement focus-aware keymap and app reducer.
- [ ] Seed 20–50 clearly fictional static records in memory for display only.
- [ ] Test keyboard focus, selection clamping, terminal size calculations and quit paths.

**Gate:** `cargo run` launches navigable cyberpunk UI; no SQLite required; restoring terminal verified manually.

## Phase 2 — Persistent SQLite storage

- [ ] Choose an FTS5-enabled SQLite build and verify capability on startup.
- [ ] Add versioned migrations, source/record model, import_jobs and FTS5 triggers.
- [ ] Implement repository with parameterized CRUD and source listing.
- [ ] Implement `init`, `seed`, and optional `doctor` commands.
- [ ] Provide reproducible synthetic dataset generator and deterministic fixtures.
- [ ] Test migrations, storage persistence, CRUD, FTS updates/deletes.

**Gate:** 1,000 fictional records persist across restarts and table/index invariants pass tests.

## Phase 3 — Actual search and inspection

- [ ] Implement safe plain-text query grammar and validation.
- [ ] Search via FTS5 `MATCH`, BM25 and `snippet()`.
- [ ] Add source/category filtering, blank query handling and pagination.
- [ ] Add inspector retrieval and empty/loading/error states.
- [ ] Validate search, ranking order, Unicode, filtering, deterministic pagination, invalid query handling.

**Gate:** No placeholder results in runtime UI; queries return consistent real DB data.

## Phase 4 — Data import

- [ ] JSON array and CSV import commands with mapping and limits.
- [ ] Transactional import job; count success/failure and explain validation diagnostics.
- [ ] Refresh source count and results after import/relaunch.
- [ ] Ensure duplicate IDs, malformed encoding, oversized rows and interrupted imports behave safely.

**Gate:** Sample JSON/CSV can be imported/researched; failed imports leave no misleading partial data.

## Phase 5 — Background worker, UX polish

- [ ] Move SQLite connection into a dedicated worker thread.
- [ ] Add typed request/response channels and search sequence IDs.
- [ ] Use bounded workload, progress indications and safe worker shutdown.
- [ ] Implement subtle loading/animation without blocking events.
- [ ] Check Ctrl+C, terminal resize, stale responses and long imports.

**Gate:** No UI stalls during normal 10k-record searches/imports, and stale results never overwrite recent searches.

## Phase 6 — Verification, packaging and handover

- [ ] Add integration tests and manual TUI checklist.
- [ ] Test 10k synthetic records and record actual hardware/data/latency measurements.
- [ ] Write user README, CLI usage and troubleshooting.
- [ ] Add CI: format, clippy, tests; platform matrix if feasible.
- [ ] Produce release binary instructions; document storage location and privacy.

**Gate:** Reproducible release and all MVP acceptance criteria pass.

## Post-MVP — only on explicit request

Multiple themes, advanced Boolean syntax, arbitrary SQL explorer (read-only carefully scoped), PostgreSQL, saved searches/export, FerrisFind Core adapter, remote endpoints. Do not introduce them into MVP work.
