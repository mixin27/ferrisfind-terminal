# Reference — original FerrisFind Core

This document distinguishes two related projects:

- **FerrisFind Core:** systems-oriented Rust file scanning/indexing engine (custom tokenizer, inverted index, positional postings, Boolean retrieval, phrase matching, BM25, incremental indexing, watcher, parallel indexing, persistence).
- **FerrisFind Terminal:** modern cyberpunk TUI built first on SQLite FTS5 and explicit local data imports.

The Terminal MVP must not depend on Core being implemented. In the future, a shared `SearchBackend` contract may expose `search`, `get_record`, `list_sources`, with stable ids, score semantics, filtering and error types. SQLite's BM25 lower-is-better score should be normalized at the adapter boundary if another backend uses different semantics. Don't compare uncalibrated backend scores.

The original Core specification can be kept separately as `docs/reference/FerrisFind-Project-Specification.md` if desired. It is background material, not a requirement to implement any Core-specific functionality in Terminal v1.
