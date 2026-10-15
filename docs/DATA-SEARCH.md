# Data storage and search design

## MVP data model

A source represents a single imported file/dataset (or deterministic seed dataset). A record belongs to one source. Avoid claiming semantic entity matching across sources. Prefer UUID text IDs for external record identity; keep SQLite `INTEGER PRIMARY KEY` for joins/FTS rowids.

## Proposed migration 001 — schema

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK(source_type IN ('seed','json','csv')),
    created_at TEXT NOT NULL,
    record_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS records (
    rowid INTEGER PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_records_source_category
ON records(source_id, category);

CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
    title, content, content='records', content_rowid='rowid',
    tokenize='unicode61'
);

CREATE TABLE IF NOT EXISTS import_jobs (
    id TEXT PRIMARY KEY,
    source_id TEXT REFERENCES sources(id),
    status TEXT NOT NULL,
    inserted_count INTEGER NOT NULL DEFAULT 0,
    rejected_count INTEGER NOT NULL DEFAULT 0,
    message TEXT,
    created_at TEXT NOT NULL
);
```

**Important:** With `content='records'`, FTS5 doesn't own the data. Maintain its index through triggers, and rebuild it after migrating legacy rows. Add and test triggers in the *same migration*. Example:

```sql
CREATE TRIGGER records_ai AFTER INSERT ON records BEGIN
  INSERT INTO records_fts(rowid,title,content)
  VALUES (new.rowid,new.title,new.content);
END;
CREATE TRIGGER records_ad AFTER DELETE ON records BEGIN
  INSERT INTO records_fts(records_fts,rowid,title,content)
  VALUES ('delete',old.rowid,old.title,old.content);
END;
CREATE TRIGGER records_au AFTER UPDATE OF title,content ON records BEGIN
  INSERT INTO records_fts(records_fts,rowid,title,content)
  VALUES ('delete',old.rowid,old.title,old.content);
  INSERT INTO records_fts(rowid,title,content)
  VALUES(new.rowid,new.title,new.content);
END;
```

Check SQLite FTS5 external-content trigger behavior against the target SQLite version and integration-test cascaded deletes, updates and rebuild (`INSERT INTO records_fts(records_fts) VALUES('rebuild')`). Trigger ordering for cascades is important; no orphaned FTS hits allowed.

## Canonical imported record

```json
{
  "id": "optional-valid-uuid",
  "title": "Fictional Network Analysis Report",
  "category": "document",
  "content": "Fictional sample content for a network design lab.",
  "metadata": {"department": "training"}
}
```

CSV required headers: `title,category,content`; optional `id` and arbitrary extra columns mapped under `metadata` as strings. JSON accepts an array of objects with the same fields; import streaming for large JSON files only after a bounded MVP parser is correct. Importer derives a source from the file, not from arbitrary incoming record attributes. Document mapping errors with row number.

## Import algorithm

1. Validate format, file extension only as hint, byte limit, CSV row count and field size limits.
2. Parse into validated `RecordDraft` objects; normalize only documented fields (trim required title/category; keep content as supplied; forbid blank title/category/content).
3. Generate stable fresh UUID IDs for missing IDs; validate supplied IDs and reject conflicts explicitly.
4. Begin transaction. Insert `sources`, `import_jobs`, records in bounded batches; update counters; commit as an atomic job for MVP. For files too large for one transaction, explicitly document chunked partial-success semantics before implementing.
5. FTS triggers index records. On failure rollback and report diagnostics; report inserted/rejected statistics only for committed data.
6. `doctor` validates source counts and FTS consistency; migrations use `PRAGMA user_version` or a migration tracking table.

## Search grammar (MVP)

**Safe default:** plain words are tokenized as Unicode text; treat user input as literal terms, not raw FTS5 query expression. Join escaped quoted terms with `AND`. Optional single final trailing `*` enables a prefix search only if explicitly implemented and tested. Boolean OR/NOT, grouping, raw query mode and fuzzy search are future features.

Algorithm sketch:

```text
parse(text):
  reject too-long input (>256 Unicode scalars or configured limit)
  tokenize into bounded nonempty terms (max 16)
  reject any token that cannot be safely encoded
  quote FTS phrases/terms and double embedded quote chars
  join by " AND "
  return Some(fts_expression) or None for blank query
```

Careful: `unicode61` tokenization may treat punctuation as separators; preserve Unicode words. Clearly document semantics of quoted phrases if added. Always bind the composed MATCH expression as a SQL parameter, never concatenate it into raw SQL text. User `source_id` and `category` are also bound parameters.

## Query algorithm

- If text is blank, run an ordinary deterministic record query (e.g. `ORDER BY created_at DESC, id ASC`) and bounded page; do not call FTS5 MATCH with empty text.
- If text nonblank, use `records_fts MATCH ?`, join `records.rowid=records_fts.rowid`, apply optional source/category filters, order `bm25(records_fts)` ascending then `records.id` ascending; `LIMIT/OFFSET` bounded and checked.
- Use `snippet(records_fts, 1, '[', ']', '…', 12)` for a preview; sanitize display of control codes. For record content use full record retrieval, not snippet alone.
- Rank is raw BM25 score (lower better). UI shows position and an optional raw score label, **not percentages**.
- Pagination with OFFSET is adequate for MVP at 10k rows. Prefer cursor/keyset pagination only when profiling shows necessity.

Illustrative SQL (adjust aliases/API to FTS5 constraints):

```sql
SELECT r.id, r.source_id, r.title, r.category, r.content,
       r.metadata_json,
       bm25(records_fts) AS rank,
       snippet(records_fts, 1, '[', ']', '...', 12) AS preview
FROM records_fts
JOIN records AS r ON r.rowid = records_fts.rowid
WHERE records_fts MATCH ?1
  AND (?2 IS NULL OR r.source_id = ?2)
  AND (?3 IS NULL OR r.category = ?3)
ORDER BY rank ASC, r.id ASC
LIMIT ?4 OFFSET ?5;
```

## Algorithms and complexity

| Algorithm | MVP use | Complexity / caveat |
|---|---|---|
| Tokenization | Unicode61 in SQLite FTS5 | Linear in input bytes; Unicode decisions matter |
| Inverted index | FTS5 internally | Lookup follows posting lists; do not claim constant-time search |
| BM25 relevance | Rank matching documents | Scores relative to corpus, lower is better in SQLite FTS5 |
| Filtered retrieval | source/category | SQL-indexed filters, inspect query plans |
| Batched transactional writes | Imports | O(n) record processing plus FTS index maintenance |
| Sequence-ID stale-result protection | UI worker | O(1) compare; stale queries not allowed to overwrite newer |

The **custom inverted index, positional postings, Boolean set operations, phrase adjacency, BM25 from scratch, incremental file hashing and multithreaded indexing** belong in the original FerrisFind Core specification and are not prerequisites for this TUI MVP.

## Reliability and privacy

Parameterize SQL. Reject malformed imports and pathological limits. Data remains local; no telemetry or surprise network connections. Never import credential dumps or real personal dossiers as sample data. Treat all imported text as untrusted display data (sanitize ANSI/control sequences). Backup before future destructive migrations. Test FTS consistency on insert, update, delete and source deletion.
