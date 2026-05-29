//! Unit tests for the renderer utilities (cell buffer / ANSI width).

use bubbletea_rs::renderer::cellbuf::{display_width, split_lines};

#[test]
fn plain_ascii_width() {
    assert_eq!(display_width("hello"), 5);
}

#[test]
fn csi_not_counted() {
    // Bold + text + reset
    assert_eq!(display_width("\x1b[1mhello\x1b[0m"), 5);
}

#[test]
fn osc_not_counted() {
    // Window title sequence
    assert_eq!(display_width("\x1b]0;title\x07"), 0);
}

#[test]
fn unicode_width_counted() {
    // CJK characters are double-width
    assert_eq!(display_width("日本"), 4);
}

#[test]
fn split_empty() {
    assert_eq!(split_lines(""), vec![""]);
}

#[test]
fn split_newline() {
    assert_eq!(split_lines("a\nb"), vec!["a", "b"]);
}

#[test]
fn split_trailing_newline() {
    // split('\n') on "a\nb\n" produces ["a", "b", ""] — trailing empty is expected.
    let lines = split_lines("a\nb\n");
    assert_eq!(lines, vec!["a", "b", ""]);
}
