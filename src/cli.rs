use std::path::PathBuf;

use clap::{Arg, Command, value_parser};

pub fn command() -> Command {
    Command::new("ferrisfind")
        .version(env!("CARGO_PKG_VERSION"))
        .about("FerrisFind Terminal: Phase 1 TUI with clearly fictional in-memory records")
        .after_help("Launch in a terminal to browse 24 fictional fixtures. F1 opens help; q quits outside search.\n\
                     Search, storage and import commands are not connected yet. No data files are created.")
        .arg(
            Arg::new("data-dir")
                .long("data-dir")
                .value_name("PATH")
                .value_parser(value_parser!(PathBuf))
                .help("Override the OS application-data directory; relative paths use the current directory"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_definition_is_consistent() {
        command().debug_assert();
    }

    #[cfg(unix)]
    #[test]
    fn path_preserves_non_utf8_bytes() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = OsString::from_vec(b"local-\xff".to_vec());
        let args = command()
            .try_get_matches_from([
                OsString::from("ferrisfind"),
                OsString::from("--data-dir"),
                path.clone(),
            ])
            .expect("OS paths need not be UTF-8");
        assert_eq!(
            args.get_one::<PathBuf>("data-dir"),
            Some(&PathBuf::from(path))
        );
    }
}
