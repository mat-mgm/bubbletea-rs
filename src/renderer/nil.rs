//! No-op renderer. Port of `nil_renderer.go`.

use super::Renderer;
use crate::view::View;

/// A renderer that does nothing. Used when rendering is disabled
/// (`Program::without_renderer`).
pub struct NilRenderer;

impl Renderer for NilRenderer {
    fn render(&mut self, _view: &View) {}
}
