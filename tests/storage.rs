use ferrisfind_terminal::{
    domain::persistence::{RecordDraft, SourceDraft, SourceKind},
    storage::{DATABASE_FILE, MAX_RECORD_BYTES, Repository, SCHEMA_VERSION, StorageError, seed},
};
use rusqlite::{Connection, params};
use tempfile::TempDir;
use uuid::Uuid;

fn setup() -> (TempDir, Repository) {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path()).unwrap();
    (directory, repository)
}

fn inspect(directory: &TempDir) -> Connection {
    Connection::open(directory.path().join(DATABASE_FILE)).unwrap()
}

fn source(index: u128) -> SourceDraft {
    SourceDraft {
        id: Uuid::from_u128(index),
        name: format!("Fictional source {index}"),
        kind: SourceKind::Seed,
    }
}

fn record(index: u128, source_id: Uuid) -> RecordDraft {
    RecordDraft {
        id: Uuid::from_u128(1000 + index),
        source_id,
        title: "oldtitle".into(),
        category: "fictional".into(),
        content: "Fictional oldbody café မြန်မာစာ".into(),
        metadata: serde_json::json!({"fictional": true}),
    }
}

// Trusted test terms only: no raw FTS grammar is exposed by the application.
fn hits(connection: &Connection, term: &str) -> i64 {
    connection
        .query_row(
            "SELECT count(*) FROM records_fts WHERE records_fts MATCH ?1",
            [term],
            |row| row.get(0),
        )
        .unwrap()
}

fn count(connection: &Connection, query: &str) -> i64 {
    connection.query_row(query, [], |row| row.get(0)).unwrap()
}

#[test]
fn migrations_are_repeatable_and_bundled_fts5_is_live() {
    let (directory, mut repository) = setup();
    let connection = inspect(&directory);
    assert_eq!(count(&connection, "PRAGMA user_version"), SCHEMA_VERSION);
    assert_eq!(
        count(
            &connection,
            "SELECT sqlite_compileoption_used('ENABLE_FTS5')"
        ),
        1
    );
    assert_eq!(
        count(
            &connection,
            "SELECT count(*) FROM sqlite_schema WHERE type='trigger'"
        ),
        3
    );
    let health = repository.check_health().unwrap();
    assert_eq!((health.sources, health.records), (0, 0));
    drop(repository);
    let mut reopened = Repository::open(directory.path()).unwrap();
    assert_eq!(reopened.check_health().unwrap().records, 0);
    assert_eq!(
        count(
            &connection,
            "SELECT count(*) FROM sqlite_schema WHERE type='trigger'"
        ),
        3
    );
}

#[test]
fn a_thousand_deterministic_records_persist_across_restarts() {
    let (directory, mut repository) = setup();
    assert_eq!(repository.seed(1000).unwrap().records, 1000);
    assert!(matches!(
        repository.seed(1000),
        Err(StorageError::SeedExists)
    ));
    drop(repository);
    let mut reopened = Repository::open_existing(directory.path()).unwrap();
    for expected in seed::records(1000).unwrap() {
        assert_eq!(
            reopened.get_record(expected.id).unwrap().unwrap().draft,
            expected
        );
    }
    let connection = inspect(&directory);
    assert_eq!(hits(&connection, "sample001000"), 1);
    assert_eq!(hits(&connection, "fictional"), 1000);
    assert_eq!(
        count(
            &connection,
            "SELECT inserted_count FROM import_jobs WHERE status='completed'"
        ),
        1000
    );
    assert_eq!(
        reopened
            .get_source(seed::source().id)
            .unwrap()
            .unwrap()
            .record_count,
        1000
    );
    reopened.check_health().unwrap();
    let (_second_directory, mut second) = setup();
    second.seed(1000).unwrap();
    let id = seed::records(1000).unwrap().last().unwrap().id;
    assert_eq!(
        reopened.get_record(id).unwrap().unwrap().draft,
        second.get_record(id).unwrap().unwrap().draft
    );
}

#[test]
fn record_crud_updates_fts_and_preserves_rowid_and_creation_time() {
    let (directory, mut repository) = setup();
    let first = source(1);
    let second = source(2);
    repository.create_source(&first).unwrap();
    repository.create_source(&second).unwrap();
    let mut draft = record(1, first.id);
    repository.insert_record(&draft).unwrap();
    let original = repository.get_record(draft.id).unwrap().unwrap();
    let connection = inspect(&directory);
    let rowid = count(&connection, "SELECT rowid FROM records");
    assert_eq!(hits(&connection, "oldtitle"), 1);
    assert_eq!(hits(&connection, "oldbody"), 1);
    assert_eq!(hits(&connection, "café"), 1);
    draft.title = "newtitle".into();
    draft.content = "Fictional newbody".into();
    draft.source_id = second.id;
    assert!(repository.update_record(&draft).unwrap());
    assert_eq!(hits(&connection, "oldtitle"), 0);
    assert_eq!(hits(&connection, "oldbody"), 0);
    assert_eq!(hits(&connection, "newtitle"), 1);
    assert_eq!(hits(&connection, "newbody"), 1);
    assert_eq!(count(&connection, "SELECT rowid FROM records"), rowid);
    assert_eq!(
        repository.get_record(draft.id).unwrap().unwrap().created_at,
        original.created_at
    );
    assert_eq!(
        repository
            .get_source(first.id)
            .unwrap()
            .unwrap()
            .record_count,
        0
    );
    assert_eq!(
        repository
            .get_source(second.id)
            .unwrap()
            .unwrap()
            .record_count,
        1
    );
    assert!(repository.delete_record(draft.id).unwrap());
    assert!(!repository.delete_record(draft.id).unwrap());
    assert!(repository.get_record(draft.id).unwrap().is_none());
    assert!(!repository.update_record(&draft).unwrap());
    assert_eq!(hits(&connection, "newbody"), 0);
    repository.check_health().unwrap();
}

#[test]
fn rowid_changes_still_keep_external_content_consistent() {
    let (directory, mut repository) = setup();
    repository.seed(1).unwrap();
    let connection = inspect(&directory);
    connection
        .execute("UPDATE records SET rowid = rowid + 100", [])
        .unwrap();
    assert_eq!(hits(&connection, "sample000001"), 1);
    repository.check_health().unwrap();
}

#[test]
fn source_crud_and_bounded_stable_listing() {
    let (_directory, mut repository) = setup();
    for index in 1..=3 {
        repository.create_source(&source(index)).unwrap();
    }
    assert!(
        repository
            .rename_source(source(1).id, "Fictional renamed")
            .unwrap()
    );
    assert_eq!(
        repository
            .get_source(source(1).id)
            .unwrap()
            .unwrap()
            .draft
            .name,
        "Fictional renamed"
    );
    let all = repository.list_sources(50, 0).unwrap();
    let mut pages = repository.list_sources(2, 0).unwrap();
    pages.extend(repository.list_sources(2, 2).unwrap());
    assert_eq!(all, pages);
    assert!(repository.list_sources(1, 3).unwrap().is_empty());
    for (limit, offset) in [(0, 0), (201, 0), (50, u64::MAX)] {
        assert!(matches!(
            repository.list_sources(limit, offset),
            Err(StorageError::InvalidPage)
        ));
    }
    assert!(repository.delete_source(source(1).id).unwrap());
    assert!(!repository.delete_source(source(1).id).unwrap());
    assert!(repository.get_source(source(1).id).unwrap().is_none());
    assert!(
        !repository
            .rename_source(source(1).id, "Fictional absent")
            .unwrap()
    );
}

#[test]
fn batch_duplicates_and_validation_failures_roll_back_all_rows_and_index_changes() {
    let (directory, mut repository) = setup();
    repository.create_source(&source(1)).unwrap();
    let original = record(1, source(1).id);
    repository.insert_record(&original).unwrap();
    let mut fresh = record(2, source(1).id);
    fresh.title = "uncommitted".into();
    assert!(
        repository
            .insert_records(&[fresh.clone(), original.clone()])
            .is_err()
    );
    let mut invalid = record(3, source(1).id);
    invalid.content.clear();
    assert!(
        repository
            .insert_records(&[fresh.clone(), invalid])
            .is_err()
    );
    let connection = inspect(&directory);
    assert_eq!(count(&connection, "SELECT count(*) FROM records"), 1);
    assert_eq!(hits(&connection, "uncommitted"), 0);
    assert_eq!(
        repository.get_record(original.id).unwrap().unwrap().draft,
        original
    );
    assert!(repository.get_record(fresh.id).unwrap().is_none());
    repository.check_health().unwrap();
}

#[test]
fn foreign_keys_reject_orphans_and_failed_moves_preserve_original_record() {
    let (_directory, mut repository) = setup();
    let mut draft = record(1, source(1).id);
    assert!(repository.insert_record(&draft).is_err());
    repository.create_source(&source(1)).unwrap();
    repository.insert_record(&draft).unwrap();
    draft.source_id = source(2).id;
    assert!(repository.update_record(&draft).is_err());
    assert_eq!(
        repository
            .get_record(draft.id)
            .unwrap()
            .unwrap()
            .draft
            .source_id,
        source(1).id
    );
    repository.check_health().unwrap();
}

#[test]
fn cascading_source_deletion_removes_fts_hits_and_retains_job_history() {
    let (directory, mut repository) = setup();
    repository.seed(30).unwrap();
    assert!(repository.delete_source(seed::source().id).unwrap());
    let connection = inspect(&directory);
    assert_eq!(hits(&connection, "fictional"), 0);
    assert_eq!(count(&connection, "SELECT count(*) FROM records"), 0);
    assert_eq!(
        count(
            &connection,
            "SELECT count(*) FROM import_jobs WHERE source_id IS NULL"
        ),
        1
    );
    repository.check_health().unwrap();
    repository.seed(20).unwrap();
    assert_eq!(count(&connection, "SELECT count(*) FROM import_jobs"), 2);
    assert_eq!(hits(&connection, "fictional"), 20);
}

#[test]
fn failed_seed_rolls_back_source_job_records_and_index() {
    let (directory, mut repository) = setup();
    let connection = inspect(&directory);
    connection.execute_batch("CREATE TRIGGER fail_seed BEFORE INSERT ON records
        WHEN new.title LIKE 'Fictional 000010%' BEGIN SELECT RAISE(ABORT, 'injected failure'); END;").unwrap();
    assert!(repository.seed(1000).is_err());
    for table in ["sources", "records", "import_jobs"] {
        assert_eq!(
            count(&connection, &format!("SELECT count(*) FROM {table}")),
            0
        );
    }
    assert_eq!(hits(&connection, "fictional"), 0);
    connection.execute_batch("DROP TRIGGER fail_seed").unwrap();
    assert_eq!(repository.seed(1000).unwrap().records, 1000);
}

#[test]
fn doctor_detects_missing_index_entries_and_rebuild_restores_them() {
    let (directory, mut repository) = setup();
    repository.seed(10).unwrap();
    let connection = inspect(&directory);
    connection
        .execute(
            "INSERT INTO records_fts(records_fts) VALUES ('delete-all')",
            [],
        )
        .unwrap();
    assert_eq!(hits(&connection, "fictional"), 0);
    assert!(repository.check_health().is_err());
    assert_eq!(hits(&connection, "fictional"), 0); // doctor did not repair
    repository.rebuild_index().unwrap();
    assert_eq!(hits(&connection, "fictional"), 10);
    repository.check_health().unwrap();
}

#[test]
fn seed_refuses_existing_corruption_without_repairing_or_adding_rows() {
    let (directory, mut repository) = setup();
    repository.create_source(&source(1)).unwrap();
    let original = record(1, source(1).id);
    repository.insert_record(&original).unwrap();
    let connection = inspect(&directory);
    connection
        .execute(
            "INSERT INTO records_fts(records_fts) VALUES ('delete-all')",
            [],
        )
        .unwrap();
    assert!(repository.seed(10).is_err());
    assert_eq!(hits(&connection, "oldtitle"), 0);
    assert_eq!(count(&connection, "SELECT count(*) FROM sources"), 1);
    assert_eq!(count(&connection, "SELECT count(*) FROM records"), 1);
    assert_eq!(count(&connection, "SELECT count(*) FROM import_jobs"), 0);
    assert_eq!(
        repository.get_record(original.id).unwrap().unwrap().draft,
        original
    );
}

#[test]
fn doctor_detects_counts_foreign_keys_and_missing_triggers() {
    let (directory, mut repository) = setup();
    repository.seed(1).unwrap();
    let connection = inspect(&directory);
    connection
        .execute("UPDATE sources SET record_count=10", [])
        .unwrap();
    assert!(matches!(
        repository.check_health(),
        Err(StorageError::Integrity("source record counts"))
    ));
    connection
        .execute("UPDATE sources SET record_count=1", [])
        .unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys=OFF; UPDATE records SET source_id='orphan';")
        .unwrap();
    assert!(matches!(
        repository.check_health(),
        Err(StorageError::Integrity("foreign key references"))
    ));
    connection
        .execute(
            "UPDATE records SET source_id=?1",
            [seed::source().id.to_string()],
        )
        .unwrap();
    repository.check_health().unwrap();
    connection.execute_batch("DROP TRIGGER records_au").unwrap();
    assert!(matches!(
        repository.check_health(),
        Err(StorageError::MissingSchemaObject("records_au"))
    ));
    assert!(matches!(
        Repository::open(directory.path()),
        Err(StorageError::MissingSchemaObject("records_au"))
    ));
}

#[test]
fn future_and_unrelated_databases_are_rejected_without_modification() {
    let (directory, repository) = setup();
    drop(repository);
    let connection = inspect(&directory);
    connection.pragma_update(None, "user_version", 999).unwrap();
    drop(connection);
    let before = std::fs::read(directory.path().join(DATABASE_FILE)).unwrap();
    assert!(matches!(
        Repository::open(directory.path()),
        Err(StorageError::FutureSchema(999))
    ));
    assert_eq!(
        before,
        std::fs::read(directory.path().join(DATABASE_FILE)).unwrap()
    );
    let unrelated = tempfile::tempdir().unwrap();
    let connection = inspect(&unrelated);
    connection
        .execute_batch(
            "CREATE TABLE unrelated(value TEXT); INSERT INTO unrelated VALUES ('preserve');",
        )
        .unwrap();
    drop(connection);
    let before = std::fs::read(unrelated.path().join(DATABASE_FILE)).unwrap();
    assert!(matches!(
        Repository::open(unrelated.path()),
        Err(StorageError::UnrecognizedDatabase)
    ));
    assert_eq!(
        before,
        std::fs::read(unrelated.path().join(DATABASE_FILE)).unwrap()
    );
}

#[test]
fn invalid_records_are_rejected_and_sql_looking_text_is_only_data() {
    let (directory, mut repository) = setup();
    repository.create_source(&source(1)).unwrap();
    let original = record(1, source(1).id);
    let mut bad = original.clone();
    bad.title = "\u{2003}".into();
    assert!(matches!(
        repository.insert_record(&bad),
        Err(StorageError::InvalidData(_))
    ));
    bad = original.clone();
    bad.content = "x".repeat(MAX_RECORD_BYTES + 1);
    assert!(matches!(
        repository.insert_record(&bad),
        Err(StorageError::InvalidData(_))
    ));
    bad = original.clone();
    bad.metadata = serde_json::json!([]);
    assert!(matches!(
        repository.insert_record(&bad),
        Err(StorageError::InvalidData(_))
    ));
    bad = original.clone();
    bad.category = "bad\0value".into();
    assert!(matches!(
        repository.insert_record(&bad),
        Err(StorageError::InvalidData(_))
    ));
    let mut literal = original;
    literal.title = "'); DROP TABLE sources; --".into();
    literal.metadata = serde_json::json!({"text": "မြန်မာစာ"});
    repository.insert_record(&literal).unwrap();
    assert_eq!(
        repository.get_record(literal.id).unwrap().unwrap().draft,
        literal
    );
    assert_eq!(
        count(&inspect(&directory), "SELECT count(*) FROM sources"),
        1
    );
    repository.check_health().unwrap();
}

#[test]
fn missing_doctor_target_invalid_paths_and_corrupt_files_are_non_destructive() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("missing");
    assert!(Repository::open_existing(&missing).is_err());
    assert!(!missing.exists());
    let file = directory.path().join("not-a-directory");
    std::fs::write(&file, b"keep").unwrap();
    assert!(matches!(
        Repository::open(&file),
        Err(StorageError::Directory { .. })
    ));
    let bad = directory.path().join(DATABASE_FILE);
    std::fs::write(&bad, b"not sqlite").unwrap();
    assert!(Repository::open(directory.path()).is_err());
    assert_eq!(std::fs::read(bad).unwrap(), b"not sqlite");
}

#[test]
fn seed_bounds_are_checked_before_any_writes() {
    let (directory, mut repository) = setup();
    for count in [0, 100_001, u32::MAX] {
        assert!(matches!(
            repository.seed(count),
            Err(StorageError::InvalidSeedCount)
        ));
    }
    assert_eq!(
        count(&inspect(&directory), "SELECT count(*) FROM sources"),
        0
    );
    assert_eq!(seed::records(100_000).unwrap().count(), 100_000);
}

#[test]
fn failed_update_rolls_back_index_and_counts() {
    let (directory, mut repository) = setup();
    repository.seed(1).unwrap();
    let mut draft = seed::records(1).unwrap().next().unwrap();
    let connection = inspect(&directory);
    connection
        .execute_batch(
            "CREATE TRIGGER fail_update AFTER UPDATE ON records
        BEGIN SELECT RAISE(ABORT, 'injected update failure'); END;",
        )
        .unwrap();
    draft.title = "uncommittedtitle".into();
    assert!(repository.update_record(&draft).is_err());
    assert_eq!(hits(&connection, "uncommittedtitle"), 0);
    assert_eq!(hits(&connection, "sample000001"), 1);
    connection
        .execute_batch("DROP TRIGGER fail_update")
        .unwrap();
    repository.check_health().unwrap();
    // UUID text is external identity; the FTS join key is an integer rowid.
    let stored_type: String = connection
        .query_row(
            "SELECT typeof(rowid) FROM records WHERE id=?1",
            params![draft.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored_type, "integer");
}
