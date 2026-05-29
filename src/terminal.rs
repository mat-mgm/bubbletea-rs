//! Terminal lifecycle management. Port of `tty.go`/`raw.go`/`termios_*.go`.
//!
//! Go uses `defer` to restore terminal state on exit; the Rust port uses an RAII
//! [`TerminalGuard`] whose `Drop` impl restores raw mode and leaves the alt
//! screen, so the terminal is cleaned up even on panic or early return.

use std::io::{self, IsTerminal, Write};

use crossterm::terminal;

use crate::view::MouseMode;

/// Owns terminal mode changes and restores them on drop.
pub struct TerminalGuard {
    /// Whether stdout/stdin is a TTY; when false, no mode changes are made.
    is_tty: bool,
    /// Whether raw mode is currently enabled by us.
    raw_enabled: bool,
    /// Whether we are currently in the alternate screen.
    alt_screen: bool,
    /// Current mouse reporting mode.
    mouse: MouseMode,
    /// Whether focus reporting is enabled.
    focus: bool,
    /// Whether bracketed paste is enabled.
    bracketed_paste: bool,
}

impl TerminalGuard {
    /// Create a guard. Enters raw mode if stdout is a TTY.
    pub fn new() -> io::Result<Self> {
        let is_tty = io::stdout().is_terminal() && io::stdin().is_terminal();
        let mut guard = TerminalGuard {
            is_tty,
            raw_enabled: false,
            alt_screen: false,
            mouse: MouseMode::None,
            focus: false,
            bracketed_paste: false,
        };
        if is_tty {
            terminal::enable_raw_mode()?;
            guard.raw_enabled = true;
        }
        Ok(guard)
    }

    /// Whether we're attached to a real terminal.
    pub fn is_tty(&self) -> bool {
        self.is_tty
    }

    /// Current terminal size, falling back to 80x24 when not a TTY.
    pub fn size(&self) -> (u16, u16) {
        if self.is_tty {
            terminal::size().unwrap_or((80, 24))
        } else {
            (80, 24)
        }
    }

    /// Enter or leave the alternate screen to match the desired state.
    pub fn set_alt_screen(&mut self, want: bool) -> io::Result<()> {
        if !self.is_tty || want == self.alt_screen {
            return Ok(());
        }
        let mut out = io::stdout();
        if want {
            write!(out, "\x1b[?1049h")?; // enter alt screen
        } else {
            write!(out, "\x1b[?1049l")?; // leave alt screen
        }
        out.flush()?;
        self.alt_screen = want;
        Ok(())
    }

    /// Enable/disable focus reporting (DECSET 1004) to match `want`.
    pub fn set_focus(&mut self, want: bool) -> io::Result<()> {
        if !self.is_tty || want == self.focus {
            return Ok(());
        }
        let mut out = io::stdout();
        write!(out, "\x1b[?1004{}", if want { 'h' } else { 'l' })?;
        out.flush()?;
        self.focus = want;
        Ok(())
    }

    /// Enable/disable bracketed paste (DECSET 2004) to match `want`.
    pub fn set_bracketed_paste(&mut self, want: bool) -> io::Result<()> {
        if !self.is_tty || want == self.bracketed_paste {
            return Ok(());
        }
        let mut out = io::stdout();
        write!(out, "\x1b[?2004{}", if want { 'h' } else { 'l' })?;
        out.flush()?;
        self.bracketed_paste = want;
        Ok(())
    }

    /// Set the mouse reporting mode (SGR-extended) to match `want`.
    pub fn set_mouse(&mut self, want: MouseMode) -> io::Result<()> {
        if !self.is_tty || want == self.mouse {
            return Ok(());
        }
        let mut out = io::stdout();
        // Disable whatever tracking mode is currently active.
        match self.mouse {
            MouseMode::None => {}
            MouseMode::CellMotion => write!(out, "\x1b[?1002l\x1b[?1006l")?,
            MouseMode::AllMotion => write!(out, "\x1b[?1003l\x1b[?1006l")?,
        }
        // Enable the requested mode (SGR mode 1006 for extended coordinates).
        match want {
            MouseMode::None => {}
            MouseMode::CellMotion => write!(out, "\x1b[?1002h\x1b[?1006h")?,
            MouseMode::AllMotion => write!(out, "\x1b[?1003h\x1b[?1006h")?,
        }
        out.flush()?;
        self.mouse = want;
        Ok(())
    }

    /// Restore terminal state now (idempotent). Called by `Drop` as well.
    pub fn restore(&mut self) {
        let _ = self.set_mouse(MouseMode::None);
        let _ = self.set_focus(false);
        let _ = self.set_bracketed_paste(false);
        if self.alt_screen {
            let mut out = io::stdout();
            let _ = write!(out, "\x1b[?1049l");
            let _ = out.flush();
            self.alt_screen = false;
        }
        if self.raw_enabled {
            let _ = terminal::disable_raw_mode();
            self.raw_enabled = false;
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore();
    }
}
