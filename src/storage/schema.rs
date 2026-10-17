use rusqlite::{Connection, TransactionBehavior};

use super::StorageError;

pub const SCHEMA_VERSION: i64 = 1;
const APPLICATION_ID: i64 = 0x4646544d; // FFTM

pub fn verify_fts5(connection: &Connection) -> Result<(), StorageError> {
    let enabled: bool = connection.query_row(
        "SELECT sqlite_compileoption_used('ENABLE_FTS5')",
        [],
        |row| row.get(0),
    )?;
    if !enabled {
        return Err(StorageError::Fts5Unavailable);
    }
    connection
        .execute_batch(
            "CREATE VIRTUAL TABLE temp.fts5_probe USING fts5(value);
         INSERT INTO temp.fts5_probe(value) VALUES ('probe');",
        )
        .map_err(|_| StorageError::Fts5Unavailable)?;
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM temp.fts5_probe WHERE fts5_probe MATCH ?1",
            ["probe"],
            |row| row.get(0),
        )
        .map_err(|_| StorageError::Fts5Unavailable)?;
    connection.execute_batch("DROP TABLE temp.fts5_probe")?;
    if count != 1 {
        return Err(StorageError::Fts5Unavailable);
    }
    Ok(())
}

pub fn migrate(connection: &mut Connection, allow_initialize: bool) -> Result<(), StorageError> {
    // Read the version under the same write lock as the migration, so concurrent
    // initializers cannot both observe an empty database and apply migration 001.
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = transaction.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(StorageError::FutureSchema(version));
    }
    let application_id: i64 =
        transaction.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    match version {
        0 if allow_initialize && application_id == 0 => {
            let objects: i64 = transaction.query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )?;
            if objects != 0 {
                return Err(StorageError::UnrecognizedDatabase);
            }
            transaction.execute_batch(include_str!("../../migrations/001_initial.sql"))?;
            transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        SCHEMA_VERSION if application_id == APPLICATION_ID => {}
        _ => return Err(StorageError::UnrecognizedDatabase),
    }
    transaction.commit()?;
    Ok(())
}

pub fn verify_objects(connection: &Connection) -> Result<(), StorageError> {
    for (kind, name) in [
        ("table", "sources"),
        ("table", "records"),
        ("table", "records_fts"),
        ("table", "import_jobs"),
        ("index", "idx_records_source_category"),
        ("trigger", "records_ai"),
        ("trigger", "records_ad"),
        ("trigger", "records_au"),
    ] {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = ?1 AND name = ?2)",
            [kind, name],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(StorageError::MissingSchemaObject(name));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_migration_rolls_back_schema_and_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA max_page_count = 2")
            .unwrap();
        assert!(migrate(&mut connection, true).is_err());
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        let application: i64 = connection
            .query_row("PRAGMA application_id", [], |row| row.get(0))
            .unwrap();
        let objects: i64 = connection
            .query_row("SELECT count(*) FROM sqlite_schema", [], |row| row.get(0))
            .unwrap();
        assert_eq!((version, application, objects), (0, 0, 0));
        connection
            .execute_batch("PRAGMA max_page_count = 1000")
            .unwrap();
        migrate(&mut connection, true).unwrap();
    }
}
