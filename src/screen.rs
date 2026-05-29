//! Screen-level commands. Port of `screen.go`.

use crate::command::{cmd, Cmd};
use crate::message::{msg, ClearScreenMsg};

/// Command that clears the screen on the next render.
///
/// Analogue of `tea.ClearScreen`.
pub fn clear_screen() -> Cmd {
    cmd(async { Some(msg(ClearScreenMsg)) })
}

