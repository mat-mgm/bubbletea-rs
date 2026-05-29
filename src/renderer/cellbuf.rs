//! ANSI-aware string utilities for the renderer.
//!
//! The view content is ANSI-encoded. We need display-width measurement that
//! ignores escape sequences, and a way to split content into logical lines.

use unicode_width::UnicodeWidthChar;

/// Measure the display width of a string, skipping ANSI escape sequences.
pub fn display_width(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut width = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        // Detect ESC start.
        if bytes[i] == b'\x1b' {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            // CSI: ESC [
            if bytes[i] == b'[' {
                i += 1;
                // Skip params and intermediates until a final byte (0x40–0x7E).
                while i < bytes.len() && !(0x40..=0x7E).contains(&bytes[i]) {
                    i += 1;
                }
                i += 1; // consume final byte
            } else if bytes[i] == b']' {
                // OSC: ESC ] … ST (BEL or ESC \)
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\x07' {
                        i += 1;
                        break;
                    }
                    if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            } else {
                // Other two-byte sequences (e.g. ESC c, ESC M).
                i += 1;
            }
            continue;
        }
        // Normal character: decode UTF-8.
        let ch = s[i..].chars().next().unwrap_or('\0');
        if ch == '\r' || ch == '\n' {
            // Don't count newlines as width.
        } else {
            width += ch.width().unwrap_or(0);
        }
        i += ch.len_utf8();
    }
    width
}

/// Split content into lines at `\n`, stripping trailing `\r`.
pub fn split_lines(content: &str) -> Vec<&str> {
    content
        .split('\n')
        .map(|l| l.trim_end_matches('\r'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_width() {
        assert_eq!(display_width("hello"), 5);
    }

    #[test]
    fn ansi_ignored() {
        assert_eq!(display_width("\x1b[31mred\x1b[0m"), 3);
    }

    #[test]
    fn osc_ignored() {
        assert_eq!(display_width("\x1b]0;title\x07hello"), 5);
    }
}
