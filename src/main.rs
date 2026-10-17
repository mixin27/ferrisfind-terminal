mod app;
mod cli;
mod config;
mod error;
mod tui;

use std::io::{self, Write};
use std::process::ExitCode;

use app::storage_commands::{self, StorageCommand};
use error::AppError;
use ferrisfind_terminal::{domain, storage::DATABASE_FILE};

fn main() -> ExitCode {
    let args = cli::command().get_matches();
    let result = config::data_dir(
        args.get_one::<std::path::PathBuf>("data-dir").cloned(),
        |key| std::env::var_os(key),
    )
    .and_then(|data_dir| {
        let command = match args.subcommand() {
            Some(("init", _)) => StorageCommand::Init,
            Some(("seed", options)) => StorageCommand::Seed {
                count: options.get_one::<u32>("count").copied()
                    .unwrap_or(ferrisfind_terminal::storage::seed::DEFAULT_SEED_COUNT),
            },
            Some(("doctor", _)) => StorageCommand::Doctor,
            _ => return app::run(domain::fictional_records()),
        };
        let report = storage_commands::execute(command, data_dir.clone())?;
        let mut output = io::stdout().lock();
        if let Some(count) = report.inserted {
            writeln!(output, "Inserted {count} fictional records (seed v1).")
                .map_err(AppError::Output)?;
        }
        writeln!(output,
            "Database: {:?}\nSQLite {} | schema {} | FTS5 verified\n{} sources | {} records | integrity OK",
            data_dir.join(DATABASE_FILE), report.health.sqlite_version,
            report.health.schema_version, report.health.sources, report.health.records,
        ).map_err(AppError::Output)
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Reporting must not panic if stderr is also closed.
            let _ = error.report(&mut io::stderr().lock());
            ExitCode::FAILURE
        }
    }
}
