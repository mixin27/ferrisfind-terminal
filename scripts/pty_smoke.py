"""Unix-only PTY smoke checks; run after `cargo build` (Python standard library)."""

import fcntl
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import termios
import time


def read_for(fd, duration=0.15):
    output = b""
    tail = b""
    deadline = time.monotonic() + duration
    while time.monotonic() < deadline:
        ready, _, _ = select.select([fd], [], [], max(0, deadline - time.monotonic()))
        if ready:
            chunk = os.read(fd, 65536)
            output += chunk
            # A PTY is not a terminal emulator. Answer cursor-position queries
            # made by Crossterm during initialization, as a real terminal would.
            if b"\x1b[6n" in tail + chunk:
                os.write(fd, b"\x1b[1;1R")
            tail = chunk[-3:]
    return output


def check(scenario):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 130, 0, 0))
    before = termios.tcgetattr(slave)
    env = dict(os.environ, TERM="xterm-256color", COLORTERM="truecolor")
    env.pop("NO_COLOR", None)
    binary = Path(__file__).resolve().parents[1] / "target/debug/ferrisfind"
    proc = subprocess.Popen(
        [str(binary), "--data-dir", "./phase1-demo"],
        stdin=slave, stdout=slave, stderr=slave, env=env, start_new_session=True,
    )
    try:
        output = b""
        deadline = time.monotonic() + 5
        while b"FICTIONAL" not in output and time.monotonic() < deadline:
            output += read_for(master, 0.25)
        output += read_for(master)
        assert b"\x1b[?1049h" in output and b"FICTIONAL" in output, output
        assert not termios.tcgetattr(slave)[3] & termios.ICANON
        assert not read_for(master, 0.25), "idle redraw detected"

        def send(data):
            os.write(master, data)
            return read_for(master)

        if scenario == "resize-and-q":
            output += send(b"/q")
            assert proc.poll() is None, "q in search quit"
            output += send(b"\x1b[200~" + "cafe\u0301 မြန်မာ 🦀".encode() + b"\x1b[201~")
            output += send(b"\r\r")
            for cols, rows in [(100, 24), (70, 20), (60, 16), (40, 10), (130, 30)]:
                fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
                os.kill(proc.pid, signal.SIGWINCH)
                output += read_for(master)
                assert proc.poll() is None
            assert b"Resize to at least" in output
            output += send(b"\x1bOP")
            output += send(b"q")
            assert proc.poll() is None, "help leaked q"
            output += send(b"\x1bOP")
            output += send(b"q")
        elif scenario == "ctrl-c-search":
            output += send(b"/q")
            output += send(b"\x03")
        else:
            os.kill(proc.pid, getattr(signal, scenario))
        output += read_for(master, 0.4)
        assert proc.wait(timeout=3) == 0, output[-500:]
        assert termios.tcgetattr(slave) == before, "terminal attributes changed"
        for sequence in [b"\x1b[?1049l", b"\x1b[?25h", b"\x1b[?2004l"]:
            assert sequence in output, sequence
        print(f"{scenario}: PASS (raw mode, idle behavior, terminal restoration)")
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        os.close(master)
        os.close(slave)


if __name__ == "__main__":
    for scenario in ["resize-and-q", "ctrl-c-search", "SIGINT", "SIGTERM", "SIGHUP"]:
        check(scenario)

    # Separate opt-in Cargo test: expected panic output must follow restoration.
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    proc = subprocess.Popen(
        ["cargo", "test", "--offline", "real_tty_error_and_panic_cleanup", "--",
         "--ignored", "--nocapture", "--test-threads=1"],
        stdin=slave, stdout=slave, stderr=slave,
        cwd=Path(__file__).resolve().parents[1], start_new_session=True,
    )
    try:
        output = b""
        deadline = time.monotonic() + 30
        while proc.poll() is None and time.monotonic() < deadline:
            output += read_for(master)
        output += read_for(master)
        assert proc.wait(timeout=3) == 0, output
        after = termios.tcgetattr(slave)
        # macOS may set PENDIN when canonical input is restored. This is a
        # transient kernel "retype pending input" state, not a raw/echo setting.
        pending = getattr(termios, "PENDIN", 0)
        before[3] &= ~pending
        after[3] &= ~pending
        assert before == after, "terminal configuration changed after error/panic"
        assert b"1 passed" in output
        assert 0 <= output.find(b"\x1b[?1049l") < output.find(b"simulated TUI panic")
        assert b"\x1b[?25h" in output and b"\x1b[?2004l" in output
        print("error-and-panic: PASS (cleanup precedes diagnostics; terminal configuration restored)")
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        os.close(master)
        os.close(slave)
