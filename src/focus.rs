//! Focus and paste events. Port of `focus.go` and `paste.go`.

/// The terminal gained focus. Analogue of `FocusMsg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusMsg;

/// The terminal lost focus. Analogue of `BlurMsg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlurMsg;

/// Pasted text received via bracketed paste. Analogue of `PasteMsg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteMsg(pub String);

impl std::fmt::Display for PasteMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
