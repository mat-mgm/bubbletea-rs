//! Line-diffing terminal renderer. Port of `cursed_renderer.go`.
//!
//! Tracks the last-rendered frame as a list of raw ANSI lines. On each flush
//! it moves the cursor back to the frame origin and rewrites only changed lines,
//! erasing stale trailing lines. In alt-screen mode it homes the cursor and
//! fills the viewport.

use std::io::{self, Write};

use super::Renderer;
use super::cellbuf::split_lines;
use crate::view::View;

/// ANSI: erase to end of line.
const ERASE_EOL: &str = "\x1b[K";
/// ANSI: erase from cursor to end of screen.
const ERASE_DOWN: &str = "\x1b[J";
/// ANSI: move to top-left (home).
const HOME: &str = "\x1b[H";
/// ANSI: cursor up N lines.
fn cursor_up(n: u16) -> String {
    if n == 0 {
        String::new()
    } else {
        format!("\x1b[{n}A")
    }
}

/// The diff renderer state.
pub struct StandardRenderer {
    /// Number of lines in the last rendered frame (inline mode).
    last_line_count: u16,
    /// Raw ANSI content of each line in the last rendered frame.
    last_lines: Vec<String>,
    /// True when the current view is in alt-screen mode.
    alt_screen: bool,
    /// Terminal width (columns).
    width: u16,
    /// Terminal height (rows).
    height: u16,
    /// Pending view (set by `render`, consumed by `flush`).
    pending: Option<View>,
    /// Whether a full erase (resize or mode change) was requested.
    force_repaint: bool,
}

impl StandardRenderer {
    pub fn new(width: u16, height: u16) -> Self {
        StandardRenderer {
            last_line_count: 0,
            last_lines: Vec::new(),
            alt_screen: false,
            width,
            height,
            pending: None,
            force_repaint: false,
        }
    }

    /// Write a complete frame in inline (non-alt-screen) mode. Only changed
    /// lines are rewritten; unchanged lines are skipped over.
    fn flush_inline(&mut self, out: &mut impl Write, lines: Vec<String>, force: bool) {
        let new_count = lines.len() as u16;
        let old_count = self.last_line_count;

        // Move cursor back to frame top.
        if old_count > 0 {
            // Move up old_count - 1 lines (we're already on the last line).
            let up = old_count.saturating_sub(1);
            let _ = write!(out, "\r{}", cursor_up(up));
        } else {
            let _ = write!(out, "\r");
        }

        // Write new lines, skipping unchanged ones.
        let empty = String::new();
        for (i, new_line) in lines.iter().enumerate() {
            let old_line = self.last_lines.get(i).unwrap_or(&empty);
            if !force && new_line == old_line {
                // Unchanged — move down to the next line.
                if i + 1 < lines.len() {
                    let _ = write!(out, "\n");
                }
                continue;
            }
            let _ = write!(out, "\r{new_line}{ERASE_EOL}");
            if i + 1 < lines.len() {
                let _ = write!(out, "\n");
            }
        }

        // Erase any lines below the new frame that the old frame had.
        if new_count < old_count {
            let extra = old_count - new_count;
            for _ in 0..extra {
                let _ = write!(out, "\n{ERASE_EOL}\r");
            }
            // Move back up.
            let _ = write!(out, "{}", cursor_up(extra));
        }

        self.last_line_count = new_count;
        self.last_lines = lines;
    }

    /// Write a complete frame in alt-screen mode (always full repaint).
    fn flush_altscreen(&mut self, out: &mut impl Write, lines: Vec<String>, force: bool) {
        let new_count = lines.len() as u16;
        let empty = String::new();

        // Home cursor.
        let _ = write!(out, "{HOME}");

        let rows = self.height;
        for row in 0..rows {
            let idx = row as usize;
            let new_line = lines.get(idx).unwrap_or(&empty);
            let old_line = self.last_lines.get(idx).unwrap_or(&empty);
            if !force && new_line == old_line {
                // Skip to next row.
                let _ = write!(out, "\r\n");
                continue;
            }
            let _ = write!(out, "\r{new_line}{ERASE_EOL}");
            if row + 1 < rows {
                let _ = write!(out, "\n");
            }
        }

        self.last_line_count = new_count.min(rows);
        self.last_lines = lines;
    }

    fn do_flush(&mut self, view: View) {
        let mut out = io::stdout();
        let force = self.force_repaint;
        self.force_repaint = false;

        // Handle alt-screen transition.
        if view.alt_screen != self.alt_screen {
            if view.alt_screen {
                let _ = write!(out, "\x1b[?1049h"); // enter alt screen
            } else {
                let _ = write!(out, "\x1b[?1049l\r"); // leave alt screen
                self.last_lines.clear();
                self.last_line_count = 0;
            }
            self.alt_screen = view.alt_screen;
        }

        // Set window title when present.
        if !view.window_title.is_empty() {
            let _ = write!(out, "\x1b]0;{}\x07", view.window_title);
        }

        // Progress bar (OSC 9;4 — Windows Terminal).
        if let Some(pb) = &view.progress_bar {
            use crate::view::ProgressBarState;
            let seq = match pb.state {
                ProgressBarState::None => "\x1b]9;4;0;0\x07".to_string(),
                ProgressBarState::Default => format!("\x1b]9;4;1;{}\x07", pb.value),
                ProgressBarState::Error => format!("\x1b]9;4;2;{}\x07", pb.value),
                ProgressBarState::Indeterminate => "\x1b]9;4;3;0\x07".to_string(),
                ProgressBarState::Warning => format!("\x1b]9;4;4;{}\x07", pb.value),
            };
            let _ = write!(out, "{seq}");
        }

        // Split content into lines.
        let lines: Vec<String> = split_lines(&view.content)
            .into_iter()
            .map(|s| s.to_owned())
            .collect();

        // Hide cursor during update to reduce flicker.
        let has_cursor = view.cursor.is_some();
        let _ = write!(out, "\x1b[?25l"); // hide cursor

        if view.alt_screen {
            self.flush_altscreen(&mut out, lines, force);
        } else {
            self.flush_inline(&mut out, lines, force);
        }

        // Render cursor if specified.
        if let Some(cur) = &view.cursor {
            let shape = match cur.shape {
                crate::cursor::CursorShape::Block => 1,
                crate::cursor::CursorShape::Underline => 3,
                crate::cursor::CursorShape::Bar => 5,
            };
            let shape = if cur.blink { shape + 1 } else { shape };
            let _ = write!(out, "\x1b[{shape} q");
            // Position cursor.
            let _ = write!(out, "\x1b[{};{}H", cur.position.y + 1, cur.position.x + 1);
            let _ = write!(out, "\x1b[?25h"); // show cursor
        } else if !has_cursor {
            let _ = write!(out, "\x1b[?25l"); // keep hidden
        }

        let _ = out.flush();
    }
}

impl Renderer for StandardRenderer {
    fn render(&mut self, view: &View) {
        // Shallow-clone the view for deferred flushing.
        self.pending = Some(View {
            content: view.content.clone(),
            cursor: view.cursor,
            alt_screen: view.alt_screen,
            report_focus: view.report_focus,
            disable_bracketed_paste: view.disable_bracketed_paste,
            mouse_mode: view.mouse_mode,
            window_title: view.window_title.clone(),
            foreground_color: view.foreground_color,
            background_color: view.background_color,
            progress_bar: view.progress_bar,
            keyboard_enhancements: view.keyboard_enhancements,
        });
    }

    fn flush(&mut self, _final_frame: bool) {
        if let Some(view) = self.pending.take() {
            self.do_flush(view);
        }
    }

    fn resize(&mut self, width: u16, height: u16) {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.force_repaint = true;
        }
    }

    fn insert_above(&mut self, line: &str) {
        // In inline mode: scroll up by inserting lines above the current frame.
        // In alt-screen mode: no-op (same as Go).
        if self.alt_screen {
            return;
        }
        let mut out = io::stdout();
        let old_count = self.last_line_count;

        // Move to frame top.
        if old_count > 0 {
            let up = old_count.saturating_sub(1);
            let _ = write!(out, "\r{}", cursor_up(up));
        } else {
            let _ = write!(out, "\r");
        }

        // Insert lines, then move cursor back to frame top.
        let n = line.matches('\n').count() as u16 + 1;
        let _ = write!(out, "\x1b[{n}L"); // insert lines
        let _ = write!(out, "{line}\r\n");

        let _ = out.flush();
        self.force_repaint = true;
    }

    fn clear_screen(&mut self) {
        let mut out = io::stdout();
        let _ = write!(out, "{ERASE_DOWN}");
        let _ = out.flush();
        self.last_lines.clear();
        self.last_line_count = 0;
        self.force_repaint = true;
    }

    fn close(&mut self) {
        let mut out = io::stdout();
        if self.alt_screen {
            let _ = write!(out, "\x1b[?1049l");
            self.alt_screen = false;
        }
        // Show cursor, clear any remaining frame artifacts.
        let _ = write!(out, "\x1b[?25h{ERASE_DOWN}");
        let _ = out.flush();
    }
}
