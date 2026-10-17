use std::path::PathBuf;

use clap::{Arg, Command, value_parser};

pub fn command() -> Command {
    Command::new("ferrisfind")
        .version(env!("CARGO_PKG_VERSION"))
        .about("FerrisFind Terminal: Phase 2 local SQLite storage and fictional-data TUI")
        .after_help("init/seed/doctor work without a terminal.\n\
                     Bare launch still shows 24 in-memory fictional fixtures; persisted data is not yet connected to the TUI.")
        .arg(
            Arg::new("data-dir")
                .long("data-dir")
                .global(true)
                .value_name("PATH")
                .value_parser(value_parser!(PathBuf))
                .help("Override the OS application-data directory; relative paths use the current directory"),
        )
        .subcommand(Command::new("init").about("Initialize or verify the local SQLite database"))
        .subcommand(Command::new("seed").about("Create one deterministic fictional dataset (refuses duplicate seeds)")
            .arg(Arg::new("count").long("count").value_name("COUNT").default_value("1000")
                .value_parser(value_parser!(u32).range(1..=100_000))
                .help("Number of fictional records, from 1 to 100000")))
        .subcommand(Command::new("doctor").about("Check an existing database and FTS index without repairing them"))
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
