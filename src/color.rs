//! Color profile and degradation. Minimal in Phase 0; expanded in Phase 5.
//!
//! Port of the `colorprofile` dependency used by Go bubbletea.

/// Terminal color capability, from most to least capable.
///
/// Analogue of `colorprofile.Profile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Message announcing the detected (or forced) color profile to the program.
#[derive(Debug, Clone, Copy)]
pub struct ColorProfileMsg(pub ColorProfile);
