# Product requirements — FerrisFind Terminal

## Goal

Make local structured data enjoyable to explore through a fast, credible **modern cyberpunk TUI** with real full-text search. Target: curious developers learning Rust, not cybersecurity exploitation.

## Primary scenarios

**S1 — Discover:** Launch TUI, view sources/status, type `/` to focus search, enter words, press Enter, browse ranked hits.  
**S2 — Inspect:** Select hit with arrows or j/k (when not typing), Enter to open an inspector; Esc returns to results.  
**S3 — Filter:** Choose a source and optional category; search results are restricted accordingly; clear filters to reset.  
**S4 — Import:** Run `ferrisfind import --file ...`; receive explicit added/skipped/failed counts and actionable diagnostics.  
**S5 — Maintain:** Run `ferrisfind doctor` to check database and FTS integrity; quit TUI without altering terminal state.

## Must-have requirements

- Works offline on macOS/Linux/Windows terminals supported by Crossterm; test the actual platforms available. Graceful fallback for low color/Unicode capability where practical.
- TUI launches with empty database; offers instructions to seed or import rather than implying data exists.
- Real search against locally stored imported/seeding data, not simulated responses.
- JSON arrays and CSV with a documented canonical mapping: required `title`, `category`, `content`; optional `id`, `metadata`, `source`. IDs may be generated; source name derives from import filename unless overridden.
- Source/category filtering, bounded result pagination, selected-row inspector, result snippets, deterministic tie-breaking.
- Import errors clearly distinguish malformed input, invalid mapping, database problems and duplicates.
- `--data-dir` override; no app secrets; no required accounts or internet.
- Visible loading, error, ready and empty states; a help overlay accessible by F1.

## Keymap (focus-aware)

| Input | Search field focused | Results focused | Inspector / help |
|---|---|---|---|
| Printable characters | Insert text | `q` quits, `/` focuses search, `j/k` navigates | Context-dependent, no input leakage |
| Enter | Execute search | Open highlighted record | Close or confirm where applicable |
| Up/Down | Optionally history (if implemented) | Move selection | Scroll inspector |
| Tab / Shift+Tab | Next / previous focus | Next / previous focus | Next / previous focus |
| Esc | Unfocus/cancel edit | Clear overlay / return | Close overlay |
| Ctrl+L | Clear search | Clear search | Clear search |
| Ctrl+R | Refresh source counters | Refresh counters | Refresh counters |
| F1 | Help | Help | Close help |
| Ctrl+C | Request graceful exit | Request graceful exit | Request graceful exit |

No global `q` shortcut while editing text. Keymaps are configuration constants, not duplicated across widgets.

## Defaults

- Search results: 50/page, hard cap 200.
- Search executes on Enter; no backend request on every keystroke in MVP.
- Sort: FTS5 BM25 ascending, followed by stable ID ascending; empty query lists recent records in a defined deterministic order instead of passing an empty `MATCH`.
- History: in-memory for the current session, up to 50 distinct commands (optional after search works).
- Import size limits should be configurable; sensible initial choices are 50 MiB/file, 1 MiB/record, 100,000 records/job, with explicit diagnostics and tests.

## Acceptance tests

1. Import at least 10,000 fictional records and perform full-text searches without locking the UI.
2. A matching record appears for known seeded terms; an unrelated term yields an empty state.
3. Filters and pagination return expected stable subsets; selection remains in bounds.
4. Editing/updating/deleting a record yields no stale FTS hits.
5. Malformed import cannot silently corrupt or partially overwrite valid records.
6. Resizing and Ctrl+C cannot leave raw mode/alternate screen enabled during normal recoverable paths.
7. TUI presents helpful offline startup and empty database instructions.

## Non-goals for MVP

Networking/remote databases; identity resolution; arbitrary cross-dataset entity correlation; graph relationships; permissions/authentication; AI reasoning; query operators beyond a defined safe subset; saving sensitive datasets in sample fixtures.
