use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("could not create data directory {path:?}: {source}")]
    Directory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("this SQLite build does not provide working FTS5 support")]
    Fts5Unavailable,
    #[error("database schema version {0} is newer than this application supports")]
    FutureSchema(i64),
    #[error("database is not a recognized FerrisFind database; it was not migrated")]
    UnrecognizedDatabase,
    #[error("required database schema object is missing: {0}")]
    MissingSchemaObject(&'static str),
    #[error("invalid record or source: {0}")]
    InvalidData(&'static str),
    #[error("page limit must be 1..=200 and offset must fit SQLite's signed integer range")]
    InvalidPage,
    #[error("seed count must be 1..=100000")]
    InvalidSeedCount,
    #[error("fictional seed v1 already exists; use a different --data-dir for another demo")]
    SeedExists,
    #[error("database integrity check failed: {0}")]
    Integrity(&'static str),
}
