//! Keyboard input. Port of `key.go` / `keyboard.go`.
//!
//! crossterm parses escape sequences into [`crossterm::event::KeyEvent`]s; this
//! module maps those into Bubble Tea's [`KeyPressMsg`] / [`KeyReleaseMsg`] with a
//! [`std::fmt::Display`] that produces the familiar keystroke strings
//! ("ctrl+c", "enter", "shift+enter", "a", ...).

use std::fmt;

use crossterm::event::{KeyCode as CtCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Modifier keys held during a key event.
///
/// Analogue of `KeyMod`. Display order is fixed: ctrl, alt, shift, meta, hyper,
/// super — so you always see "ctrl+shift+alt+a", never a reordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyMod {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub hyper: bool,
    pub super_: bool,
}

impl KeyMod {
    /// True if any modifier other than shift is held (used to decide whether a
    /// printable key renders as its text or as a keystroke).
    fn has_non_shift(self) -> bool {
        self.ctrl || self.alt || self.meta || self.hyper || self.super_
    }

    fn write_prefix(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            f.write_str("ctrl+")?;
        }
        if self.alt {
            f.write_str("alt+")?;
        }
        if self.shift {
            f.write_str("shift+")?;
        }
        if self.meta {
            f.write_str("meta+")?;
        }
        if self.hyper {
            f.write_str("hyper+")?;
        }
        if self.super_ {
            f.write_str("super+")?;
        }
        Ok(())
    }
}

impl From<KeyModifiers> for KeyMod {
    fn from(m: KeyModifiers) -> Self {
        KeyMod {
            ctrl: m.contains(KeyModifiers::CONTROL),
            alt: m.contains(KeyModifiers::ALT),
            shift: m.contains(KeyModifiers::SHIFT),
            meta: m.contains(KeyModifiers::META),
            hyper: m.contains(KeyModifiers::HYPER),
            super_: m.contains(KeyModifiers::SUPER),
        }
    }
}

/// A key code: either a printable character or a named special key.
///
/// Analogue of `Key.Code`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Backspace,
    Enter,
    Tab,
    BackTab,
    Esc,
    Space,
    /// Function key F1..=F24.
    F(u8),
    /// Any key crossterm reported that has no dedicated variant here.
    Other,
}

impl KeyCode {
    /// The keystroke name of the key, without modifiers (e.g. "enter", "f1", "a").
    fn name(self) -> String {
        match self {
            KeyCode::Char(' ') | KeyCode::Space => "space".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Up => "up".into(),
            KeyCode::Down => "down".into(),
            KeyCode::Left => "left".into(),
            KeyCode::Right => "right".into(),
            KeyCode::Home => "home".into(),
            KeyCode::End => "end".into(),
            KeyCode::PageUp => "pgup".into(),
            KeyCode::PageDown => "pgdown".into(),
            KeyCode::Insert => "insert".into(),
            KeyCode::Delete => "delete".into(),
            KeyCode::Backspace => "backspace".into(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Tab => "tab".into(),
            KeyCode::BackTab => "shift+tab".into(),
            KeyCode::Esc => "esc".into(),
            KeyCode::F(n) => format!("f{n}"),
            KeyCode::Other => "unknown".into(),
        }
    }
}

impl From<CtCode> for KeyCode {
    fn from(c: CtCode) -> Self {
        match c {
            CtCode::Char(ch) => KeyCode::Char(ch),
            CtCode::Up => KeyCode::Up,
            CtCode::Down => KeyCode::Down,
            CtCode::Left => KeyCode::Left,
            CtCode::Right => KeyCode::Right,
            CtCode::Home => KeyCode::Home,
            CtCode::End => KeyCode::End,
            CtCode::PageUp => KeyCode::PageUp,
            CtCode::PageDown => KeyCode::PageDown,
            CtCode::Insert => KeyCode::Insert,
            CtCode::Delete => KeyCode::Delete,
            CtCode::Backspace => KeyCode::Backspace,
            CtCode::Enter => KeyCode::Enter,
            CtCode::Tab => KeyCode::Tab,
            CtCode::BackTab => KeyCode::BackTab,
            CtCode::Esc => KeyCode::Esc,
            CtCode::F(n) => KeyCode::F(n),
            _ => KeyCode::Other,
        }
    }
}

/// A key press or release event.
///
/// Analogue of Go's `Key`. Match on `key.to_string()` for convenience, or on
/// `key.code` for robustness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyMod,
    /// Printable text for the key, when it represents printable character(s).
    pub text: String,
    /// Whether this is part of a key-repeat sequence (needs kbd enhancements).
    pub is_repeat: bool,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Mirror Go's `Key.String`: printable text wins unless a non-shift
        // modifier is held (then we render the keystroke, e.g. "ctrl+c").
        if let KeyCode::Char(_) = self.code {
            if !self.modifiers.has_non_shift() && !self.text.is_empty() {
                return f.write_str(&self.text);
            }
        }
        self.modifiers.write_prefix(f)?;
        f.write_str(&self.code.name())
    }
}

impl Key {
    fn from_event(ev: &KeyEvent) -> Self {
        let code = KeyCode::from(ev.code);
        let modifiers = KeyMod::from(ev.modifiers);
        // Populate text only for printable characters without a non-shift mod.
        let text = match code {
            KeyCode::Char(c) if !modifiers.has_non_shift() => c.to_string(),
            _ => String::new(),
        };
        Key {
            code,
            modifiers,
            text,
            is_repeat: ev.kind == KeyEventKind::Repeat,
        }
    }
}

/// A key press event. Analogue of `KeyPressMsg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPressMsg(pub Key);

/// A key release event (requires keyboard enhancements). Analogue of `KeyReleaseMsg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyReleaseMsg(pub Key);

impl fmt::Display for KeyPressMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for KeyReleaseMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Convert a crossterm key event into a press/release message.
///
/// Returns `None` for event kinds we don't translate. Press and Repeat both map
/// to [`KeyPressMsg`] (with `is_repeat` set for repeats), matching Bubble Tea.
pub(crate) fn from_crossterm(ev: KeyEvent) -> Option<crate::message::Msg> {
    let key = Key::from_event(&ev);
    match ev.kind {
        KeyEventKind::Press | KeyEventKind::Repeat => {
            Some(crate::message::msg(KeyPressMsg(key)))
        }
        KeyEventKind::Release => Some(crate::message::msg(KeyReleaseMsg(key))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers as M;

    fn press(code: CtCode, mods: M) -> String {
        let ev = KeyEvent::new(code, mods);
        Key::from_event(&ev).to_string()
    }

    #[test]
    fn plain_char_renders_as_text() {
        assert_eq!(press(CtCode::Char('a'), M::NONE), "a");
        assert_eq!(press(CtCode::Char('Q'), M::SHIFT), "Q");
    }

    #[test]
    fn ctrl_alt_render_as_keystroke() {
        assert_eq!(press(CtCode::Char('c'), M::CONTROL), "ctrl+c");
        assert_eq!(press(CtCode::Char('a'), M::ALT), "alt+a");
        assert_eq!(
            press(CtCode::Char('a'), M::CONTROL | M::ALT),
            "ctrl+alt+a"
        );
    }

    #[test]
    fn modifier_order_is_fixed() {
        // ctrl, alt, shift — regardless of how they combine.
        assert_eq!(
            press(CtCode::Char('a'), M::SHIFT | M::ALT | M::CONTROL),
            "ctrl+alt+shift+a"
        );
    }

    #[test]
    fn special_keys() {
        assert_eq!(press(CtCode::Enter, M::NONE), "enter");
        assert_eq!(press(CtCode::Esc, M::NONE), "esc");
        assert_eq!(press(CtCode::Tab, M::NONE), "tab");
        assert_eq!(press(CtCode::BackTab, M::NONE), "shift+tab");
        assert_eq!(press(CtCode::F(5), M::NONE), "f5");
        assert_eq!(press(CtCode::Up, M::NONE), "up");
        assert_eq!(press(CtCode::PageDown, M::NONE), "pgdown");
    }

    #[test]
    fn special_keys_with_modifiers() {
        assert_eq!(press(CtCode::Enter, M::SHIFT), "shift+enter");
        assert_eq!(press(CtCode::Left, M::CONTROL), "ctrl+left");
    }

    #[test]
    fn space_is_named() {
        // Plain space renders as its text " " (matches Go's Key.String); with a
        // non-shift modifier it falls back to the keystroke name "space".
        assert_eq!(press(CtCode::Char(' '), M::NONE), " ");
        assert_eq!(press(CtCode::Char(' '), M::CONTROL), "ctrl+space");
    }
}
