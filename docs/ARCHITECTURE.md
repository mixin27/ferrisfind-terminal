# Architecture and module contracts

## MVP decision

Single Cargo package. Rust std + Ratatui/Crossterm for TUI, SQLite via `rusqlite` with FTS5, Serde/CSV for imports. Start synchronously with mock data for Phase 1; once SQLite enters, move DB calls to a dedicated worker thread. Do not add Tokio or Axum in the MVP.

## Dependency direction

```text
main / CLI assembly
   ├── tui (rendering and input mapping only)
   ├── app (state, actions, orchestration)
   │     └── domain (types, validation, contracts)
   └── infrastructure (search/sqlite, storage, importer)
              └── domain
```

No reverse imports from domain to infrastructure/TUI; no database connection inside AppState; no SQL in rendering functions. `main` is the composition root.

## Proposed modules

```text
src/
  main.rs
  app/{mod,state,action,reducer,event_loop}.rs
  domain/{mod,record,query,search}.rs
  search/{mod,sqlite}.rs
  storage/{mod,connection,schema,repository}.rs
  importer/{mod,json,csv}.rs
  tui/{mod,layout,theme,input}.rs
  tui/screens/{search,inspector,help}.rs
  tui/widgets/{header,results,search_bar,status_bar}.rs
  error.rs
migrations/
sample-data/
tests/
```

Use the smallest structure needed at each phase; these are target module boundaries, not an instruction to create every empty file in advance.

### Implemented through Phase 1

`main` resolves CLI configuration and composes the application. `domain.rs` holds
the display record shape and 24 static fictional fixtures. `app.rs` owns state and
the pure action reducer; `app/event_loop.rs` owns the single event loop.
`tui/` holds key mapping/help, layout, theme, rendering, the terminal guard and
Unix interrupt notifications. Rendering accepts immutable app state; ephemeral
Ratatui list state only scrolls the view to the reducer's selection.
Before rendering, the event loop updates the inspector's scroll bound from the
wrapped line count and current size. No search/storage/importer modules exist yet.

## Domain shapes (illustrative, not obligatory exact API)

```rust
struct Record {
    id: String,
    source_id: String,
    title: String,
    category: String,
    content: String,
    metadata: serde_json::Value,
}
struct SearchQuery {
    text: String,
    source_id: Option<String>,
    category: Option<String>,
    limit: u32,
    offset: u64,
}
struct SearchHit {
    record: Record,
    rank: f64, // SQLite BM25: smaller is better
    snippet: String,
}
trait SearchBackend {
    fn search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>, SearchError>;
    fn get_record(&self, id: &str) -> Result<Option<Record>, SearchError>;
}
```

`SearchBackend` may live only inside the worker initially; trait object sharing across threads is not required. Keep domain abstractions ergonomic; add `Send` bounds only where actually needed.

## Event-driven behavior

1. Crossterm produces keyboard/resize events.
2. Input mapper dispatches typed `Action` (e.g., `SearchSubmitted`, `ResultSelected`, `OpenInspector`, `QuitRequested`).
3. Reducer updates `AppState` and optionally emits a typed effect (e.g., `WorkerCommand::Search`).
4. Worker owns SQLite connection and sends back `WorkerEvent::SearchCompleted { request_id, ... }`.
5. Reducer rejects stale `request_id`, updates result state, clamps selected index.
6. Ratatui draws `&AppState`; rendering never mutates DB or blocks on I/O.

```text
TUI event loop --Action--> AppState reducer
                        |          ^
                      Effect       | WorkerEvent
                        v          |
                 bounded channel -> DB worker -> SQLite
```

## Worker lifecycle

- Dedicated DB worker for all search and import operations; create the connection inside that thread.
- Commands: `Search`, `GetRecord`, `Import`, `ListSources`, `CheckHealth`, `Shutdown`; responses include matching IDs, structured error, or progress.
- Search sequence counter increments on each submitted query; apply responses only if current.
- On quit: stop accepting commands, request shutdown, restore terminal reliably even if worker is unavailable; avoid indefinite blocking during thread join.
- If an import is long-running, report progress at coarse intervals and allow safe cancellation between batches (MVP may limit import to one job at a time).
- SQLite write contention: one connection/worker serializes operations. Don't wrap it in shared `Arc<Mutex<Connection>>` for the TUI.

## Terminal lifecycle

Create an RAII guard for raw mode, alternate screen, cursor hide/show, bracketed paste if enabled. Install a panic hook only if it safely chains and restores terminal. Ensure error and early-return paths restore state. Use stdout only through Ratatui while active; send logs to file/stderr before entering alternate screen or after leaving.

## Configuration

CLI flags take precedence over optional config; defaults work offline. Data dir OS-specific, override `--data-dir` for dev/tests. Put migrations under version control. Do not use environment variables for fake credentials.

## Future adapters

A future FerrisFind Core implementation can satisfy a backend contract without changing UI and app action contracts. Postgres and remote endpoints belong to separate later decisions. Avoid prematurely writing generic federation abstractions.
