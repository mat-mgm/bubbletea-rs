//! A simple stdout renderer used until the diff renderer (Phase 3) lands.
//!
//! It repaints the whole frame each render by erasing the previous one with ANSI
//! cursor movement. Not optimized — just enough to run programs end-to-end.

use std::io::{self, Write};

use super::Renderer;
use crate::view::View;

/// Naive full-repaint renderer writing to stdout.
pub struct PlainRenderer {
    last_lines: u16,
}

impl PlainRenderer {
    pub fn new() -> Self {
        PlainRenderer { last_lines: 0 }
    }
}

impl Default for PlainRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for PlainRenderer {
    fn render(&mut self, view: &View) {
        let mut out = io::stdout().lock();
        // Erase the previously drawn frame: move up and clear each line.
        for _ in 0..self.last_lines {
            let _ = write!(out, "\x1b[1A\x1b[2K");
        }
        let _ = write!(out, "\r");
        let content = &view.content;
        let _ = out.write_all(content.as_bytes());
        if !content.ends_with('\n') {
            let _ = out.write_all(b"\n");
        }
        self.last_lines = content.lines().count() as u16;
        let _ = out.flush();
    }

    fn insert_above(&mut self, line: &str) {
        let mut out = io::stdout().lock();
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }

    fn clear_screen(&mut self) {
        let mut out = io::stdout().lock();
        let _ = write!(out, "\x1b[2J\x1b[H");
        self.last_lines = 0;
        let _ = out.flush();
    }

    fn close(&mut self) {
        let mut out = io::stdout().lock();
        let _ = out.flush();
    }
}
