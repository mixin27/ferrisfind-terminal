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
        for expected in ["ferrisfind", "--data-dir", "--help", "--version", "Phase 0"] {
            assert!(help.contains(expected), "missing {expected}: {help}");
        }
        for future_command in ["init", "seed", "import", "doctor"] {
            assert!(!help.contains(&format!("  {future_command} ")));
        }
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
        vec!["init"],
        vec!["seed"],
        vec!["import"],
        vec!["doctor"],
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
fn explicit_paths_work_without_environment_defaults() {
    for args in [
        vec!["--data-dir", "local data/မြန်မာ"],
        vec!["--data-dir=local data/မြန်မာ"],
    ] {
        let output = command().args(args).output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let escaped_path = format!("{:?}", std::path::Path::new("local data/မြန်မာ"));
        assert!(stdout(&output).contains(&escaped_path));
        assert!(stdout(&output).contains("not implemented yet"));
        assert!(!output.stdout.contains(&0x1b));
    }
}

#[test]
fn startup_and_help_do_not_create_data_directory() {
    let path = std::env::temp_dir().join(format!("ferrisfind-phase0-{}", std::process::id()));
    assert!(!path.exists(), "test requires a nonexistent path");
    for flag in [None, Some("--help"), Some("--version")] {
        let mut process = command();
        process.arg("--data-dir").arg(&path);
        if let Some(flag) = flag {
            process.arg(flag);
        }
        let output = process.output().unwrap();
        assert!(output.status.success());
        assert!(!path.exists());
    }
}
