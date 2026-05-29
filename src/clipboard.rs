//! Clipboard commands via OSC 52. Port of `clipboard.go`.

use crate::command::{cmd, Cmd};
use crate::message::msg;

/// A clipboard read response (OSC 52 reply from the terminal).
///
/// Analogue of `tea.ClipboardMsg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardMsg {
    /// Pasted content decoded from the OSC 52 response.
    pub content: String,
    /// Selection type: `b'c'` for system clipboard, `b'p'` for primary.
    pub selection: u8,
}

impl std::fmt::Display for ClipboardMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content)
    }
}

// --- Internal message types --------------------------------------------------

pub(crate) struct SetClipboardMsg(pub(crate) String);
pub(crate) struct ReadClipboardMsg;
pub(crate) struct SetPrimaryClipboardMsg(pub(crate) String);
pub(crate) struct ReadPrimaryClipboardMsg;

// --- Commands ----------------------------------------------------------------

/// Set the system clipboard using OSC 52. Analogue of `tea.SetClipboard`.
pub fn set_clipboard(s: impl Into<String> + Send + 'static) -> Cmd {
    cmd(async move { Some(msg(SetClipboardMsg(s.into()))) })
}

/// Request the system clipboard content via OSC 52. Analogue of `tea.ReadClipboard`.
/// The terminal responds with a [`ClipboardMsg`].
pub fn read_clipboard() -> Cmd {
    cmd(async { Some(msg(ReadClipboardMsg)) })
}

/// Set the primary (X11/Wayland) clipboard. Analogue of `tea.SetPrimaryClipboard`.
pub fn set_primary_clipboard(s: impl Into<String> + Send + 'static) -> Cmd {
    cmd(async move { Some(msg(SetPrimaryClipboardMsg(s.into()))) })
}

/// Request the primary clipboard content. Analogue of `tea.ReadPrimaryClipboard`.
pub fn read_primary_clipboard() -> Cmd {
    cmd(async { Some(msg(ReadPrimaryClipboardMsg)) })
}
