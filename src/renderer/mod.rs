//! Renderer abstraction. Port of `renderer.go`'s `renderer` interface.
//!
//! Phase 0/1 ship [`NilRenderer`] and a simple [`PlainRenderer`]. The diff-based
//! cell renderer (port of `cursed_renderer.go`) lands in Phase 3 as
//! `StandardRenderer`.

mod nil;
mod plain;

pub use nil::NilRenderer;
pub use plain::PlainRenderer;

use crate::view::View;

/// A renderer turns [`View`]s into terminal output.
///
/// Analogue of Go's unexported `renderer` interface. Methods that are only
/// meaningful for the full renderer have default no-op implementations so the
/// simpler renderers don't need to implement them.
pub trait Renderer: Send {
    /// Render a view to the terminal.
    fn render(&mut self, view: &View);

    /// Notify the renderer of a new terminal size.
    fn resize(&mut self, _width: u16, _height: u16) {}

    /// Print a line above the managed output (Println/Printf).
    fn insert_above(&mut self, _line: &str) {}

    /// Clear the screen.
    fn clear_screen(&mut self) {}

    /// Flush any buffered output. `final_frame` marks the last flush on shutdown.
    fn flush(&mut self, _final_frame: bool) {}

    /// Tear down the renderer, restoring terminal state as needed.
    fn close(&mut self) {}
}
