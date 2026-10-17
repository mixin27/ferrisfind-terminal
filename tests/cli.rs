use std::process::{Command, Output};

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ferrisfind"));
    // Child-only environment changes avoid global mutation and real user defaults.
    for key in ["HOME", "LOCALAPPDATA", "XDG_DATA_HOME"] {
        command.env_remove(key);
    }
    command
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 output")
}

#[test]
fn help_and_version_work_without_tty_or_data_directory() {
    for flag in ["--help", "-h"] {
        let output = command().arg(flag).output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let help = stdout(&output);
        for expected in [
            "ferrisfind",
            "--data-dir",
            "--help",
            "--version",
            "Phase 2",
            "init",
            "seed",
            "doctor",
        ] {
            assert!(help.contains(expected), "missing {expected}: {help}");
        }
        assert!(!help.contains("  import "));
    }
    for flag in ["--version", "-V"] {
        let output = command().arg(flag).output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(
            stdout(&output).trim(),
            concat!("ferrisfind ", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn invalid_arguments_fail_with_usage_error() {
    for args in [
        vec!["--unknown"],
        vec!["--data-dir"],
        vec!["--data-dir="],
        vec!["--data-dir", "a", "--data-dir", "b"],
        vec!["seed", "--count", "0"],
        vec!["seed", "--count", "100001"],
        vec!["seed", "--count", "-1"],
        vec!["seed", "--count", "abc"],
        vec!["seed", "--count"],
        vec!["import"],
        vec!["init", "--count", "1"],
    ] {
        let output = command().args(&args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("error:"));
    }
}

#[test]
fn unavailable_default_is_actionable_and_structured() {
    let output = command().output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.starts_with("level=error event=configuration_failed message="));
    assert!(error.contains("--data-dir PATH"));
}

#[test]
fn explicit_paths_reach_tty_validation_without_environment_defaults() {
    for args in [
        vec!["--data-dir", "local data/မြန်မာ"],
        vec!["--data-dir=local data/မြန်မာ"],
    ] {
        let output = command().args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("event=terminal_failed"));
        assert!(error.contains("requires a terminal"));
        assert!(!error.contains('\x1b'));
    }
}

#[test]
fn startup_and_help_do_not_create_data_directory() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("not-created");
    assert!(!path.exists(), "test requires a nonexistent path");
    for flag in [None, Some("--help"), Some("--version")] {
        let mut process = command();
        process.arg("--data-dir").arg(&path);
        if let Some(flag) = flag {
            process.arg(flag);
        }
        let output = process.output().unwrap();
        assert_eq!(output.status.success(), flag.is_some());
        assert!(!path.exists());
    }
}

#[test]
fn init_seed_and_doctor_work_without_a_tty_and_persist_between_processes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fictional data မြန်မာ");
    for _ in 0..2 {
        let output = command()
            .arg("init")
            .arg("--data-dir")
            .arg(&path)
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output);
        assert!(stdout(&output).contains("0 sources | 0 records | integrity OK"));
        assert!(output.stderr.is_empty());
    }
    let output = command()
        .arg("--data-dir")
        .arg(&path)
        .arg("seed")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert!(stdout(&output).contains("Inserted 1000 fictional records"));
    assert!(!output.stdout.contains(&0x1b));
    let output = command()
        .arg("doctor")
        .arg("--data-dir")
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert!(stdout(&output).contains("1 sources | 1000 records | integrity OK"));
    assert!(stdout(&output).contains("FTS5 verified"));
    let output = command()
        .arg("seed")
        .arg("--data-dir")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
    let mut repository = ferrisfind_terminal::storage::Repository::open_existing(&path).unwrap();
    assert_eq!(repository.check_health().unwrap().records, 1000);
}

#[test]
fn subcommand_help_and_invalid_counts_do_not_create_storage() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("absent");
    for subcommand in ["init", "seed", "doctor"] {
        let output = command()
            .arg(subcommand)
            .arg("--data-dir")
            .arg(&path)
            .arg("--help")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!path.exists());
    }
    let output = command()
        .args(["seed", "--count", "100001", "--data-dir"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!path.exists());
    let output = command()
        .args(["doctor", "--data-dir"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(!path.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("event=storage_failed"));
}

#[test]
fn doctor_reports_corruption_without_repair_and_invalid_paths_fail_cleanly() {
    use ferrisfind_terminal::storage::{DATABASE_FILE, Repository};
    let directory = tempfile::tempdir().unwrap();
    let mut repository = Repository::open(directory.path()).unwrap();
    repository.seed(1).unwrap();
    drop(repository);
    let connection = rusqlite::Connection::open(directory.path().join(DATABASE_FILE)).unwrap();
    connection
        .execute("UPDATE sources SET record_count=5", [])
        .unwrap();
    let output = command()
        .arg("doctor")
        .arg("--data-dir")
        .arg(directory.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("source record counts"));
    assert_eq!(
        connection
            .query_row("SELECT record_count FROM sources", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        5
    );
    let file = directory.path().join("file");
    std::fs::write(&file, b"keep").unwrap();
    let output = command()
        .args(["init", "--data-dir"])
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(!output.stderr.contains(&0x1b));
    assert_eq!(std::fs::read(file).unwrap(), b"keep");
}

#[test]
fn concurrent_seeds_commit_exactly_one_dataset() {
    use std::process::Stdio;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("concurrent");
    let spawn = || {
        command()
            .args(["seed", "--count", "1000", "--data-dir"])
            .arg(&path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    };
    let first = spawn();
    let second = spawn();
    let outputs = [
        first.wait_with_output().unwrap(),
        second.wait_with_output().unwrap(),
    ];
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.status.success())
            .count(),
        1,
        "{outputs:?}"
    );
    let failed = outputs
        .iter()
        .find(|output| !output.status.success())
        .unwrap();
    assert!(
        String::from_utf8_lossy(&failed.stderr).contains("already exists"),
        "{failed:?}"
    );
    let mut repository = ferrisfind_terminal::storage::Repository::open_existing(&path).unwrap();
    assert_eq!(repository.check_health().unwrap().records, 1000);
}
