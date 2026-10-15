mod cli;
mod config;
mod error;

use std::io::{self, Write};
use std::process::ExitCode;

use error::AppError;

fn main() -> ExitCode {
    let args = cli::command().get_matches();
    let result = config::data_dir(
        args.get_one::<std::path::PathBuf>("data-dir").cloned(),
        |key| std::env::var_os(key),
    )
    .and_then(|data_dir| write_status(&mut io::stdout().lock(), &data_dir));

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Reporting must not panic if stderr is also closed.
            let _ = error.report(&mut io::stderr().lock());
            ExitCode::FAILURE
        }
    }
}

fn write_status(output: &mut impl Write, data_dir: &std::path::Path) -> Result<(), AppError> {
    writeln!(
        output,
        "FerrisFind Terminal — Phase 0 CLI foundation.\n\
         The interactive TUI is not implemented yet.\n\
         Data directory (not created): {data_dir:?}"
    )
    .map_err(AppError::Output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_failure_is_a_typed_error() {
        struct ClosedOutput;
        impl Write for ClosedOutput {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let error = write_status(&mut ClosedOutput, std::path::Path::new("data"))
            .expect_err("closed output must fail");
        assert!(
            matches!(error, AppError::Output(ref source) if source.kind() == io::ErrorKind::BrokenPipe)
        );
        assert!(std::error::Error::source(&error).is_some());
    }
}
