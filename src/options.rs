//! Program configuration. Port of `options.go` (functional options) as a builder.

use crate::color::ColorProfile;

/// Configuration knobs for a [`crate::Program`], set via builder methods.
///
/// Each field corresponds to a Go `WithX` option.
pub struct Options {
    /// Disable all input (Go `WithInput(nil)` / `disableInput`).
    pub disable_input: bool,
    /// Handle OS signals ourselves instead of via the runtime (Go `WithoutSignalHandler`).
    pub disable_signal_handler: bool,
    /// Disable panic catching (Go `WithoutCatchPanics`).
    pub disable_catch_panics: bool,
    /// Ignore OS signals entirely (Go `WithoutSignals`, used in tests).
    pub ignore_signals: bool,
    /// Disable the renderer; output is sent plainly (Go `WithoutRenderer`).
    pub disable_renderer: bool,
    /// Max renderer FPS; clamped to [1, 120] with a default of 60 (Go `WithFPS`).
    pub fps: u16,
    /// Forced color profile, if any (Go `WithColorProfile`).
    pub color_profile: Option<ColorProfile>,
    /// Initial window size override (Go `WithWindowSize`).
    pub window_size: Option<(u16, u16)>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            disable_input: false,
            disable_signal_handler: false,
            disable_catch_panics: false,
            ignore_signals: false,
            disable_renderer: false,
            fps: super::DEFAULT_FPS,
            color_profile: None,
            window_size: None,
        }
    }
}
