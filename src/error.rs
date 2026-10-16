use std::error::Error;
use std::fmt;
use std::io::{self, Write};

#[derive(Debug)]
pub enum AppError {
    EmptyDataDir,
    MissingDataDir,
    NotATerminal,
    Terminal(io::Error),
}

impl AppError {
    /// Phase 0 diagnostics use one escaped key/value record per stderr line.
    /// Call only outside an active alternate screen; never log imported content.
    pub fn report(&self, output: &mut impl Write) -> io::Result<()> {
        let event = match self {
            Self::EmptyDataDir | Self::MissingDataDir => "configuration_failed",
            Self::NotATerminal | Self::Terminal(_) => "terminal_failed",
        };
        writeln!(
            output,
            "level=error event={event} message={:?}",
            self.to_string()
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDataDir => write!(f, "--data-dir must not be empty"),
            Self::MissingDataDir => write!(
                f,
                "cannot determine an absolute application-data directory; pass --data-dir PATH"
            ),
            Self::NotATerminal => write!(
                f,
                "interactive launch requires a terminal on stdin and stdout; use --help for CLI usage"
            ),
            Self::Terminal(source) => write!(f, "terminal operation failed: {source}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Terminal(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_escape_control_characters_into_one_line() {
        let error = AppError::Terminal(io::Error::other("bad\n\x1b[31m\"output\""));
        let mut output = Vec::new();
        error.report(&mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.starts_with("level=error event=terminal_failed message="));
        assert_eq!(text.lines().count(), 1);
        assert!(!text.contains('\x1b'));
        assert!(text.contains("\\n"));
    }
}
