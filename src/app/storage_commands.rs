use std::{path::PathBuf, sync::mpsc, thread};

use ferrisfind_terminal::storage::{HealthReport, Repository, StorageError};

use crate::error::AppError;

#[derive(Clone, Copy)]
pub enum StorageCommand {
    Init,
    Seed { count: u32 },
    Doctor,
}

pub struct StorageReport {
    pub health: HealthReport,
    pub inserted: Option<u32>,
}

/// A bounded, one-response CLI worker. No SQLite call runs on the TUI event loop.
pub fn execute(command: StorageCommand, data_dir: PathBuf) -> Result<StorageReport, AppError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker = thread::Builder::new()
        .name("ferrisfind-db".into())
        .spawn(move || {
            let result = run(command, data_dir);
            let _ = sender.send(result);
        })
        .map_err(AppError::WorkerStart)?;
    let result = receiver.recv();
    worker
        .join()
        .map_err(|_| AppError::Worker("database worker panicked"))?;
    result
        .map_err(|_| AppError::Worker("database worker closed without a response"))?
        .map_err(AppError::Storage)
}

fn run(command: StorageCommand, data_dir: PathBuf) -> Result<StorageReport, StorageError> {
    let mut repository = match command {
        StorageCommand::Doctor => Repository::open_existing(&data_dir)?,
        _ => Repository::open(&data_dir)?,
    };
    if let StorageCommand::Seed { count } = command {
        Ok(StorageReport {
            health: repository.seed(count)?,
            inserted: Some(count),
        })
    } else {
        Ok(StorageReport {
            health: repository.check_health()?,
            inserted: None,
        })
    }
}
