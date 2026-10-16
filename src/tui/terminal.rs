use std::io::{self, IsTerminal};

use crossterm::{
    cursor::{Hide, Show},
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::error::AppError;

// This small boundary makes setup/cleanup failures testable without a real TTY.
trait Control {
    fn raw(&mut self, enabled: bool) -> io::Result<()>;
    fn screen(&mut self, enabled: bool) -> io::Result<()>;
}

pub struct SystemTerminal;

impl Control for SystemTerminal {
    fn raw(&mut self, enabled: bool) -> io::Result<()> {
        if enabled {
            terminal::enable_raw_mode()
        } else {
            terminal::disable_raw_mode()
        }
    }

    fn screen(&mut self, enabled: bool) -> io::Result<()> {
        if enabled {
            execute!(
                io::stdout(),
                EnterAlternateScreen,
                EnableBracketedPaste,
                Hide
            )
        } else {
            // Attempt every restoration operation even if an earlier write fails.
            let paste = execute!(io::stdout(), DisableBracketedPaste);
            let cursor = execute!(io::stdout(), Show);
            let screen = execute!(io::stdout(), LeaveAlternateScreen);
            paste.and(cursor).and(screen)
        }
    }
}

struct Guard<C: Control> {
    control: C,
    raw: bool,
    screen: bool,
}

impl<C: Control> Guard<C> {
    fn enter(control: C) -> io::Result<Self> {
        let mut guard = Self {
            control,
            raw: false,
            screen: false,
        };
        // Mark cleanup needed before fallible operations: writes may partially succeed.
        guard.raw = true;
        guard.control.raw(true)?;
        guard.screen = true;
        guard.control.screen(true)?;
        Ok(guard)
    }

    fn restore(&mut self) -> io::Result<()> {
        let screen = if self.screen {
            self.control.screen(false)
        } else {
            Ok(())
        };
        if screen.is_ok() {
            self.screen = false;
        }
        let raw = if self.raw {
            self.control.raw(false)
        } else {
            Ok(())
        };
        if raw.is_ok() {
            self.raw = false;
        }
        screen.and(raw)
    }
}

impl<C: Control> Drop for Guard<C> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

pub fn install_panic_hook() {
    // The binary installs this once. Restore before chaining diagnostics from the
    // previous hook; Drop alone would run after the panic message was printed.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = SystemTerminal.screen(false);
        let _ = SystemTerminal.raw(false);
        previous_hook(info);
    }));
}

pub fn with_terminal(run: impl FnOnce() -> io::Result<()>) -> Result<(), AppError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(AppError::NotATerminal);
    }
    let mut guard = Guard::enter(SystemTerminal).map_err(AppError::Terminal)?;
    let result = run();
    let restored = guard.restore();
    // Both operations ran; preserve the original application error if there was one.
    result.and(restored).map_err(AppError::Terminal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    struct Fake {
        calls: Rc<RefCell<Vec<&'static str>>>,
        fail: Option<&'static str>,
    }
    impl Fake {
        fn call(&self, name: &'static str) -> io::Result<()> {
            self.calls.borrow_mut().push(name);
            if self.fail == Some(name) {
                Err(io::Error::other(name))
            } else {
                Ok(())
            }
        }
    }
    impl Control for Fake {
        fn raw(&mut self, enabled: bool) -> io::Result<()> {
            self.call(if enabled { "raw on" } else { "raw off" })
        }
        fn screen(&mut self, enabled: bool) -> io::Result<()> {
            self.call(if enabled { "screen on" } else { "screen off" })
        }
    }

    #[test]
    #[ignore = "requires a real TTY; run separately with --ignored --nocapture --test-threads=1"]
    fn real_tty_error_and_panic_cleanup() {
        assert!(io::stdin().is_terminal() && io::stdout().is_terminal());
        install_panic_hook();
        let error = with_terminal(|| Err(io::Error::other("simulated render error")));
        assert!(matches!(error, Err(AppError::Terminal(_))));
        assert!(!terminal::is_raw_mode_enabled().unwrap());
        let panic = std::panic::catch_unwind(|| {
            let _ = with_terminal(|| panic!("simulated TUI panic"));
        });
        assert!(panic.is_err());
        assert!(!terminal::is_raw_mode_enabled().unwrap());
    }

    #[test]
    fn partial_setup_and_scope_exit_restore_in_order() {
        for (fail, expected) in [
            (Some("raw on"), vec!["raw on", "raw off"]),
            (
                Some("screen on"),
                vec!["raw on", "screen on", "screen off", "raw off"],
            ),
            (None, vec!["raw on", "screen on", "screen off", "raw off"]),
        ] {
            let calls = Rc::new(RefCell::new(vec![]));
            let guard = Guard::enter(Fake {
                calls: calls.clone(),
                fail,
            });
            assert_eq!(guard.is_err(), fail.is_some());
            drop(guard);
            assert_eq!(*calls.borrow(), expected);
        }
    }

    #[test]
    fn cleanup_error_does_not_skip_raw_mode_and_drop_retries() {
        let calls = Rc::new(RefCell::new(vec![]));
        let mut guard = Guard::enter(Fake {
            calls: calls.clone(),
            fail: Some("screen off"),
        })
        .unwrap();
        assert!(guard.restore().is_err());
        drop(guard);
        assert_eq!(
            *calls.borrow(),
            ["raw on", "screen on", "screen off", "raw off", "screen off"]
        );
    }

    #[test]
    fn guard_restores_on_early_error_and_panic_unwind() {
        let calls = Rc::new(RefCell::new(vec![]));
        let early_error = || -> io::Result<()> {
            let _guard = Guard::enter(Fake {
                calls: calls.clone(),
                fail: None,
            })?;
            Err(io::Error::other("render failed"))
        };
        assert!(early_error().is_err());
        assert!(calls.borrow().ends_with(&["screen off", "raw off"]));
        calls.borrow_mut().clear();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = Guard::enter(Fake {
                calls: calls.clone(),
                fail: None,
            })
            .unwrap();
            panic!("simulated failure");
        }));
        assert!(result.is_err());
        assert!(calls.borrow().ends_with(&["screen off", "raw off"]));
    }
}
