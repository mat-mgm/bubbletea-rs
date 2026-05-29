//! Color profile detection and degradation. Port of `color.go` / `profile.go`.
//!
//! The profile is detected once at startup from environment variables and
//! terminal capabilities, then broadcast as a [`ColorProfileMsg`]. Programs can
//! inspect it to generate appropriately degraded styled output.

/// Terminal color capability, from most to least capable.
///
/// Analogue of `colorprofile.Profile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ColorProfile {
    /// 24-bit truecolor.
    #[default]
    TrueColor,
    /// 256-color (ANSI 256).
    Ansi256,
    /// 16-color (ANSI).
    Ansi,
    /// No color.
    Ascii,
}

/// Detect the color profile from environment variables.
///
/// Analogue of `colorprofile.Detect`. Checks (in order):
/// 1. `NO_COLOR` → Ascii
/// 2. `COLORTERM=truecolor|24bit` → TrueColor
/// 3. `TERM_PROGRAM` heuristics
/// 4. `TERM` suffix matching
pub fn detect_profile() -> ColorProfile {
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorProfile::Ascii;
    }
    if let Ok(ct) = std::env::var("COLORTERM") {
        if ct.eq_ignore_ascii_case("truecolor") || ct.eq_ignore_ascii_case("24bit") {
            return ColorProfile::TrueColor;
        }
    }
    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        match tp.to_lowercase().as_str() {
            "iterm.app" | "hyper" | "wezterm" | "rio" => return ColorProfile::TrueColor,
            "apple_terminal" => return ColorProfile::Ansi256,
            _ => {}
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") {
            return ColorProfile::Ansi256;
        }
        if term.contains("truecolor") {
            return ColorProfile::TrueColor;
        }
        if matches!(
            term.as_str(),
            "xterm" | "vt100" | "screen" | "linux" | "ansi"
        ) {
            return ColorProfile::Ansi;
        }
        if term.starts_with("xterm") || term.starts_with("rxvt") {
            return ColorProfile::Ansi256;
        }
    }
    // Safe fallback: assume 256-color.
    ColorProfile::Ansi256
}

/// Message announcing the detected (or forced) color profile to the program.
///
/// Sent once on startup. Analogue of `tea.ColorProfileMsg`.
#[derive(Debug, Clone, Copy)]
pub struct ColorProfileMsg(pub ColorProfile);

// --- Color query messages & commands ----------------------------------------

/// A terminal background color response (OSC 11 reply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundColorMsg {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl BackgroundColorMsg {
    /// Returns true if the color's luminance is below 50% (a "dark" background).
    pub fn is_dark(self) -> bool {
        let luma = 0.2126 * (self.r as f32 / 255.0)
            + 0.7152 * (self.g as f32 / 255.0)
            + 0.0722 * (self.b as f32 / 255.0);
        luma < 0.5
    }
}

/// A terminal foreground color response (OSC 10 reply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForegroundColorMsg {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A terminal cursor color response (OSC 12 reply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorColorMsg {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub(crate) struct RequestBackgroundColorMsg;
pub(crate) struct RequestForegroundColorMsg;
pub(crate) struct RequestCursorColorMsg;

/// Command to request the terminal background color (OSC 11 query).
pub fn request_background_color() -> crate::command::Cmd {
    crate::command::cmd(async { Some(crate::message::msg(RequestBackgroundColorMsg)) })
}

/// Command to request the terminal foreground color (OSC 10 query).
pub fn request_foreground_color() -> crate::command::Cmd {
    crate::command::cmd(async { Some(crate::message::msg(RequestForegroundColorMsg)) })
}

/// Command to request the terminal cursor color (OSC 12 query).
pub fn request_cursor_color() -> crate::command::Cmd {
    crate::command::cmd(async { Some(crate::message::msg(RequestCursorColorMsg)) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_background() {
        assert!(BackgroundColorMsg { r: 0, g: 0, b: 0 }.is_dark());
    }

    #[test]
    fn light_background() {
        assert!(!BackgroundColorMsg { r: 255, g: 255, b: 255 }.is_dark());
    }
}
