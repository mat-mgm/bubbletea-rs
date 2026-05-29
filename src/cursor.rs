//! Cursor types. Port of `cursor.go`.

/// A position in the terminal (column `x`, row `y`), zero-based from the top-left
/// of the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

/// Terminal cursor shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    Bar,
}

/// A cursor on the terminal screen.
///
/// Analogue of Go's `tea.Cursor`. When a [`crate::View`] carries `Some(Cursor)`
/// the cursor is shown at the given position.
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub position: Position,
    /// Cursor color; `None` uses the terminal default.
    pub color: Option<anstyle::Color>,
    pub shape: CursorShape,
    pub blink: bool,
}

impl Cursor {
    /// New cursor at `(x, y)` with default shape, blinking, terminal-default color.
    pub fn new(x: u16, y: u16) -> Self {
        Cursor {
            position: Position { x, y },
            color: None,
            shape: CursorShape::Block,
            blink: true,
        }
    }
}

/// Reports the terminal cursor position. Sent in response to
/// [`crate::command::request_cursor_position`] (added in a later phase).
#[derive(Debug, Clone, Copy)]
pub struct CursorPositionMsg {
    pub x: u16,
    pub y: u16,
}
