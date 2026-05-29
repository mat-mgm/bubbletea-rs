//! The [`View`] type returned by `Model::view`.
//!
//! Port of Go's `tea.View`. Phase 0 carries the core fields; color, progress
//! bar, and keyboard-enhancement fields are fleshed out in later phases.

use crate::cursor::Cursor;

/// How mouse events are reported for a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseMode {
    /// Mouse events disabled.
    #[default]
    None,
    /// Click, release, wheel, and drag (motion while a button is held).
    CellMotion,
    /// All events including motion with no button held.
    AllMotion,
}

/// A terminal view: the content to render plus per-frame terminal state.
///
/// Analogue of Go's `tea.View`. Construct with [`View::new`] or [`View::default`].
#[derive(Default)]
pub struct View {
    /// Screen content: styled text with ANSI escape codes.
    pub content: String,
    /// Optional cursor; when `Some`, shown at the given position.
    pub cursor: Option<Cursor>,
    /// Put the program in the alternate screen buffer (full-window mode).
    pub alt_screen: bool,
    /// Report focus/blur events to `update`.
    pub report_focus: bool,
    /// Disable bracketed paste for this view.
    pub disable_bracketed_paste: bool,
    /// Mouse reporting mode for this view.
    pub mouse_mode: MouseMode,
    /// Terminal window title (support depends on the terminal).
    pub window_title: String,
}

impl View {
    /// Create a view with the given styled content.
    pub fn new(content: impl Into<String>) -> Self {
        View {
            content: content.into(),
            ..Default::default()
        }
    }

    /// Set the content of the view.
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }
}
