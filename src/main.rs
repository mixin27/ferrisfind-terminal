mod app;
mod cli;
mod config;
mod domain;
mod error;
mod tui;

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = cli::command().get_matches();
    let result = config::data_dir(
        args.get_one::<std::path::PathBuf>("data-dir").cloned(),
        |key| std::env::var_os(key),
    )
    .and_then(|_data_dir| app::run(domain::fictional_records()));

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Reporting must not panic if stderr is also closed.
            let _ = error.report(&mut io::stderr().lock());
            ExitCode::FAILURE
        }
    }
}
