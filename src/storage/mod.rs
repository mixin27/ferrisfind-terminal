mod error;
mod schema;
pub mod seed;

pub use error::StorageError;
pub use schema::SCHEMA_VERSION;

use std::{fs, path::Path, time::Duration};

use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, TransactionBehavior, params};
use uuid::Uuid;

use crate::domain::persistence::{RecordDraft, Source, SourceDraft, SourceKind, StoredRecord};

pub const DATABASE_FILE: &str = "ferrisfind.sqlite3";
pub const MAX_RECORD_BYTES: usize = 1024 * 1024;
pub const DEFAULT_PAGE_SIZE: u32 = 50;
pub const MAX_PAGE_SIZE: u32 = 200;

#[derive(Debug)]
pub struct HealthReport {
    pub sqlite_version: String,
    pub schema_version: i64,
    pub sources: u64,
    pub records: u64,
}

/// The runtime creates and uses this exclusively on its database worker thread.
/// Tests can own isolated repositories directly. The connection is never shared.
pub struct Repository {
    connection: Connection,
}

impl Repository {
    /// Initialize an empty database or open an existing FerrisFind database.
    pub fn open(data_dir: &Path) -> Result<Self, StorageError> {
        Self::connect(data_dir, true)
    }

    /// Open an existing database without creating directories or applying migrations.
    pub fn open_existing(data_dir: &Path) -> Result<Self, StorageError> {
        Self::connect(data_dir, false)
    }

    fn connect(data_dir: &Path, create: bool) -> Result<Self, StorageError> {
        if create {
            fs::create_dir_all(data_dir).map_err(|source| StorageError::Directory {
                path: data_dir.into(),
                source,
            })?;
        }
        // Deliberately omit SQLITE_OPEN_URI: user-selected paths are filesystem paths.
        let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        if create {
            flags |= OpenFlags::SQLITE_OPEN_CREATE;
        }
        let mut connection = Connection::open_with_flags(data_dir.join(DATABASE_FILE), flags)?;
        connection.busy_timeout(Duration::from_secs(2))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        schema::verify_fts5(&connection)?;
        schema::migrate(&mut connection, create)?;
        schema::verify_objects(&connection)?;
        Ok(Self { connection })
    }

    pub fn create_source(&mut self, source: &SourceDraft) -> Result<(), StorageError> {
        validate_source(source)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        insert_source(&tx, source)?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_source(&self, id: Uuid) -> Result<Option<Source>, StorageError> {
        Ok(self
            .connection
            .query_row(
                "SELECT id, name, source_type, created_at, record_count FROM sources WHERE id = ?1",
                [id.to_string()],
                read_source,
            )
            .optional()?)
    }

    pub fn list_sources(&self, limit: u32, offset: u64) -> Result<Vec<Source>, StorageError> {
        if !(1..=MAX_PAGE_SIZE).contains(&limit) || offset > i64::MAX as u64 {
            return Err(StorageError::InvalidPage);
        }
        let mut statement = self.connection.prepare(
            "SELECT id, name, source_type, created_at, record_count FROM sources
             ORDER BY created_at DESC, id ASC LIMIT ?1 OFFSET ?2",
        )?;
        Ok(statement
            .query_map(params![limit, offset as i64], read_source)?
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn rename_source(&mut self, id: Uuid, name: &str) -> Result<bool, StorageError> {
        if name.trim().is_empty() || name.len() > 4096 || name.contains('\0') {
            return Err(StorageError::InvalidData(
                "source name must be nonblank, NUL-free and at most 4096 bytes",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            "UPDATE sources SET name = ?1 WHERE id = ?2",
            params![name, id.to_string()],
        )?;
        tx.commit()?;
        Ok(changed == 1)
    }

    pub fn delete_source(&mut self, id: Uuid) -> Result<bool, StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute("DELETE FROM sources WHERE id = ?1", [id.to_string()])?;
        tx.commit()?;
        Ok(changed == 1)
    }

    pub fn insert_record(&mut self, record: &RecordDraft) -> Result<(), StorageError> {
        self.insert_records(std::slice::from_ref(record))
    }

    /// One transaction: an invalid/duplicate row rolls the entire batch back.
    pub fn insert_records(&mut self, records: &[RecordDraft]) -> Result<(), StorageError> {
        if records.len() > seed::MAX_SEED_COUNT as usize {
            return Err(StorageError::InvalidData(
                "record batch exceeds 100000 rows",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        for record in records {
            insert_record(&tx, record)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_record(&self, id: Uuid) -> Result<Option<StoredRecord>, StorageError> {
        Ok(self
            .connection
            .query_row(
                "SELECT id, source_id, title, category, content, metadata_json, created_at
             FROM records WHERE id = ?1",
                [id.to_string()],
                read_record,
            )
            .optional()?)
    }

    /// Preserve id/rowid/creation time, while allowing a record to move sources.
    pub fn update_record(&mut self, record: &RecordDraft) -> Result<bool, StorageError> {
        let metadata = validate_record(record)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            "UPDATE records SET source_id = ?1, title = ?2, category = ?3,
             content = ?4, metadata_json = ?5 WHERE id = ?6",
            params![
                record.source_id.to_string(),
                record.title,
                record.category,
                record.content,
                metadata,
                record.id.to_string()
            ],
        )?;
        tx.commit()?;
        Ok(changed == 1)
    }

    pub fn delete_record(&mut self, id: Uuid) -> Result<bool, StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute("DELETE FROM records WHERE id = ?1", [id.to_string()])?;
        tx.commit()?;
        Ok(changed == 1)
    }

    /// Seed source, job, rows and final index rebuild commit together.
    pub fn seed(&mut self, count: u32) -> Result<HealthReport, StorageError> {
        let records = seed::records(count)?;
        let source = seed::source();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?1)",
            [source.id.to_string()],
            |row| row.get(0),
        )?;
        if exists {
            return Err(StorageError::SeedExists);
        }
        health(&tx)?;
        insert_source(&tx, &source)?;
        let last_job: i64 = tx.query_row(
            "SELECT coalesce(max(rowid), 0) FROM import_jobs",
            [],
            |row| row.get(0),
        )?;
        let next_job = last_job
            .checked_add(1)
            .ok_or(StorageError::Integrity("job sequence exhausted"))?;
        let job_id =
            Uuid::new_v5(&source.id, format!("seed-job-v1-{next_job}").as_bytes()).to_string();
        tx.execute("INSERT INTO import_jobs(id, source_id, status, message) VALUES (?1, ?2, 'running', ?3)",
            params![job_id, source.id.to_string(), "Fictional deterministic seed v1"])?;
        for record in records {
            insert_record(&tx, &record)?;
        }
        tx.execute(
            "INSERT INTO records_fts(records_fts) VALUES ('rebuild')",
            [],
        )?;
        tx.execute(
            "UPDATE import_jobs SET status = 'completed', inserted_count = ?1 WHERE id = ?2",
            params![count, job_id],
        )?;
        let report = health(&tx)?;
        tx.commit()?;
        Ok(report)
    }

    pub fn rebuild_index(&mut self) -> Result<(), StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO records_fts(records_fts) VALUES ('rebuild')",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Validate without repairing: rank=1 compares FTS to external content too.
    pub fn check_health(&mut self) -> Result<HealthReport, StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let report = health(&tx)?;
        tx.commit()?;
        Ok(report)
    }
}

fn health(tx: &Connection) -> Result<HealthReport, StorageError> {
    schema::verify_objects(tx)?;
    let integrity: String = tx.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(StorageError::Integrity("SQLite integrity_check"));
    }
    let foreign_key_error = tx.prepare("PRAGMA foreign_key_check")?.exists([])?;
    if foreign_key_error {
        return Err(StorageError::Integrity("foreign key references"));
    }
    let bad_counts: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sources s WHERE s.record_count !=
             (SELECT count(*) FROM records r WHERE r.source_id = s.id))",
        [],
        |row| row.get(0),
    )?;
    if bad_counts {
        return Err(StorageError::Integrity("source record counts"));
    }
    tx.execute(
        "INSERT INTO records_fts(records_fts, rank) VALUES ('integrity-check', 1)",
        [],
    )?;
    let report = tx.query_row(
        "SELECT sqlite_version(), (SELECT count(*) FROM sources), (SELECT count(*) FROM records)",
        [],
        |row| {
            Ok(HealthReport {
                sqlite_version: row.get(0)?,
                schema_version: SCHEMA_VERSION,
                sources: read_count(row, 1)?,
                records: read_count(row, 2)?,
            })
        },
    )?;
    Ok(report)
}

fn validate_source(source: &SourceDraft) -> Result<(), StorageError> {
    if source.name.trim().is_empty() || source.name.len() > 4096 || source.name.contains('\0') {
        return Err(StorageError::InvalidData(
            "source name must be nonblank, NUL-free and at most 4096 bytes",
        ));
    }
    Ok(())
}

fn validate_record(record: &RecordDraft) -> Result<String, StorageError> {
    for text in [&record.title, &record.category, &record.content] {
        if text.trim().is_empty() || text.contains('\0') {
            return Err(StorageError::InvalidData(
                "title, category and content must be nonblank and NUL-free",
            ));
        }
    }
    if !record.metadata.is_object() {
        return Err(StorageError::InvalidData("metadata must be a JSON object"));
    }
    let metadata = record.metadata.to_string();
    let size = record
        .title
        .len()
        .saturating_add(record.category.len())
        .saturating_add(record.content.len())
        .saturating_add(metadata.len());
    if size > MAX_RECORD_BYTES {
        return Err(StorageError::InvalidData(
            "record text and metadata exceed 1 MiB",
        ));
    }
    Ok(metadata)
}

fn insert_source(connection: &Connection, source: &SourceDraft) -> Result<(), StorageError> {
    validate_source(source)?;
    connection.execute(
        "INSERT INTO sources(id, name, source_type) VALUES (?1, ?2, ?3)",
        params![source.id.to_string(), source.name, source.kind.as_str()],
    )?;
    Ok(())
}

fn insert_record(connection: &Connection, record: &RecordDraft) -> Result<(), StorageError> {
    let metadata = validate_record(record)?;
    connection
        .prepare_cached(
            "INSERT INTO records(id, source_id, title, category, content, metadata_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?
        .execute(params![
            record.id.to_string(),
            record.source_id.to_string(),
            record.title,
            record.category,
            record.content,
            metadata
        ])?;
    Ok(())
}

fn read_count(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let count: i64 = row.get(index)?;
    count
        .try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, count))
}

fn read_uuid(row: &Row<'_>, index: usize) -> rusqlite::Result<Uuid> {
    let value: String = row.get(index)?;
    Uuid::parse_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn read_source(row: &Row<'_>) -> rusqlite::Result<Source> {
    let kind: String = row.get(2)?;
    let kind = match kind.as_str() {
        "seed" => SourceKind::Seed,
        "json" => SourceKind::Json,
        "csv" => SourceKind::Csv,
        _ => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::other("unknown source type")),
            ));
        }
    };
    Ok(Source {
        draft: SourceDraft {
            id: read_uuid(row, 0)?,
            name: row.get(1)?,
            kind,
        },
        created_at: row.get(3)?,
        record_count: read_count(row, 4)?,
    })
}

fn read_record(row: &Row<'_>) -> rusqlite::Result<StoredRecord> {
    let metadata: String = row.get(5)?;
    let metadata = serde_json::from_str(&metadata).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(StoredRecord {
        draft: RecordDraft {
            id: read_uuid(row, 0)?,
            source_id: read_uuid(row, 1)?,
            title: row.get(2)?,
            category: row.get(3)?,
            content: row.get(4)?,
            metadata,
        },
        created_at: row.get(6)?,
    })
}
