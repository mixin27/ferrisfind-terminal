use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::AppError;

/// Resolve configuration without touching the filesystem or mutating the environment.
pub fn data_dir(
    override_dir: Option<PathBuf>,
    env: impl Fn(&str) -> Option<OsString>,
) -> Result<PathBuf, AppError> {
    if let Some(path) = override_dir {
        return if path.as_os_str().is_empty() {
            Err(AppError::EmptyDataDir)
        } else {
            Ok(path)
        };
    }

    // Empty/relative environment defaults are unusable; never silently fall back
    // to the working directory. Explicit relative CLI paths remain supported.
    let absolute_env = |key| {
        env(key)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };

    #[cfg(target_os = "windows")]
    let base = absolute_env("LOCALAPPDATA");

    #[cfg(target_os = "macos")]
    let base = absolute_env("HOME").map(|home| home.join("Library/Application Support"));

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = absolute_env("XDG_DATA_HOME")
        .or_else(|| absolute_env("HOME").map(|home| home.join(".local/share")));

    base.map(|base| base.join("ferrisfind"))
        .ok_or(AppError::MissingDataDir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_path_takes_precedence_without_reading_environment() {
        let expected = PathBuf::from("local data/မြန်မာ");
        assert_eq!(
            data_dir(Some(expected.clone()), |_| panic!("must use override")).unwrap(),
            expected
        );
    }

    #[test]
    fn empty_override_is_rejected() {
        assert!(matches!(
            data_dir(Some(PathBuf::new()), |_| None),
            Err(AppError::EmptyDataDir)
        ));
    }

    #[test]
    fn missing_empty_or_relative_environment_requires_override() {
        for value in [
            None,
            Some(OsString::new()),
            Some(OsString::from("relative")),
        ] {
            assert!(matches!(
                data_dir(None, |_| value.clone()),
                Err(AppError::MissingDataDir)
            ));
        }
    }

    #[test]
    fn platform_default() {
        #[cfg(target_os = "windows")]
        let (key, root, expected) = (
            "LOCALAPPDATA",
            r"C:\Users\fictional\AppData\Local",
            r"C:\Users\fictional\AppData\Local\ferrisfind",
        );
        #[cfg(target_os = "macos")]
        let (key, root, expected) = (
            "HOME",
            "/fictional",
            "/fictional/Library/Application Support/ferrisfind",
        );
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let (key, root, expected) = ("HOME", "/fictional", "/fictional/.local/share/ferrisfind");

        assert_eq!(
            data_dir(None, |name| (name == key).then(|| root.into())).unwrap(),
            PathBuf::from(expected)
        );
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    #[test]
    fn xdg_precedence_and_invalid_xdg_fallback() {
        for (xdg, expected) in [
            ("/custom", "/custom/ferrisfind"),
            ("", "/fictional/.local/share/ferrisfind"),
            ("relative", "/fictional/.local/share/ferrisfind"),
        ] {
            assert_eq!(
                data_dir(None, |key| match key {
                    "XDG_DATA_HOME" => Some(xdg.into()),
                    "HOME" => Some("/fictional".into()),
                    _ => None,
                })
                .unwrap(),
                PathBuf::from(expected)
            );
        }
    }
}
