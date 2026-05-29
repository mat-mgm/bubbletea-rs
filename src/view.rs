//! The [`View`] type returned by `Model::view`. Port of Go's `tea.View`.

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

/// Terminal progress bar state. Analogue of `ProgressBarState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressBarState {
    #[default]
    None,
    Default,
    Error,
    Indeterminate,
    Warning,
}

/// A terminal progress bar (Windows Terminal / OSC 9;4 sequence).
///
/// Analogue of `tea.ProgressBar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressBar {
    pub state: ProgressBarState,
    /// Progress value 0–100. Ignored for `None` and `Indeterminate`.
    pub value: u8,
}

impl ProgressBar {
    pub fn new(state: ProgressBarState, value: u8) -> Self {
        ProgressBar {
            state,
            value: value.min(100),
        }
    }
}

/// Keyboard enhancement features (Kitty keyboard protocol).
///
/// Analogue of `tea.KeyboardEnhancements`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyboardEnhancements {
    /// Report key repeat and release events.
    pub report_event_types: bool,
    /// Report alternate key values.
    pub report_alternate_keys: bool,
    /// Report all keys as escape codes (including plain text).
    pub report_all_keys_as_escape_codes: bool,
    /// Report the text associated with key events.
    pub report_associated_text: bool,
}

/// A terminal view: content + per-frame terminal state.
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
    /// Terminal window title.
    pub window_title: String,
    /// Terminal foreground color override (`None` resets to terminal default).
    pub foreground_color: Option<anstyle::Color>,
    /// Terminal background color override (`None` resets to terminal default).
    pub background_color: Option<anstyle::Color>,
    /// Terminal progress bar (Windows Terminal / OSC 9;4).
    pub progress_bar: Option<ProgressBar>,
    /// Keyboard enhancement features to request from the terminal.
    pub keyboard_enhancements: KeyboardEnhancements,
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
