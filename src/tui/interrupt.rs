//! Signal handlers only set a flag; terminal I/O stays on the event-loop thread.
use std::io;

#[cfg(unix)]
pub struct Interrupt {
    flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handlers: Vec<signal_hook::SigId>,
}

#[cfg(unix)]
impl Interrupt {
    pub fn new() -> io::Result<Self> {
        use signal_hook::consts::signal::{SIGHUP, SIGINT, SIGTERM};
        let mut interrupt = Self {
            flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            handlers: Vec::new(),
        };
        for signal in [SIGHUP, SIGINT, SIGTERM] {
            interrupt
                .handlers
                .push(signal_hook::flag::register(signal, interrupt.flag.clone())?);
        }
        Ok(interrupt)
    }

    pub fn requested(&self) -> bool {
        self.flag.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(unix)]
impl Drop for Interrupt {
    fn drop(&mut self) {
        for id in self.handlers.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}

#[cfg(not(unix))]
pub struct Interrupt;

#[cfg(not(unix))]
impl Interrupt {
    pub fn new() -> io::Result<Self> {
        Ok(Self)
    }
    pub fn requested(&self) -> bool {
        false
    }
}
