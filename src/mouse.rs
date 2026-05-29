//! Mouse input. Port of `mouse.go`.

use std::fmt;

use crossterm::event::{MouseButton as CtButton, MouseEvent, MouseEventKind};

use crate::key::KeyMod;

/// Which button was involved in a mouse event. Analogue of `MouseButton`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseButton {
    #[default]
    None,
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
    Backward,
    Forward,
}

impl MouseButton {
    fn name(self) -> &'static str {
        match self {
            MouseButton::None => "none",
            MouseButton::Left => "left",
            MouseButton::Middle => "middle",
            MouseButton::Right => "right",
            MouseButton::WheelUp => "wheelup",
            MouseButton::WheelDown => "wheeldown",
            MouseButton::WheelLeft => "wheelleft",
            MouseButton::WheelRight => "wheelright",
            MouseButton::Backward => "backward",
            MouseButton::Forward => "forward",
        }
    }
}

/// A mouse event. Coordinates are zero-based from the top-left. Analogue of `Mouse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mouse {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
    pub modifiers: KeyMod,
}

impl fmt::Display for Mouse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.ctrl {
            f.write_str("ctrl+")?;
        }
        if self.modifiers.alt {
            f.write_str("alt+")?;
        }
        if self.modifiers.shift {
            f.write_str("shift+")?;
        }
        f.write_str(self.button.name())
    }
}

/// Mouse button press. Analogue of `MouseClickMsg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseClickMsg(pub Mouse);
/// Mouse button release. Analogue of `MouseReleaseMsg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseReleaseMsg(pub Mouse);
/// Mouse wheel movement. Analogue of `MouseWheelMsg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseWheelMsg(pub Mouse);
/// Mouse motion. Analogue of `MouseMotionMsg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseMotionMsg(pub Mouse);

fn button_from(b: CtButton) -> MouseButton {
    match b {
        CtButton::Left => MouseButton::Left,
        CtButton::Right => MouseButton::Right,
        CtButton::Middle => MouseButton::Middle,
    }
}

/// Convert a crossterm mouse event into the matching message.
pub(crate) fn from_crossterm(ev: MouseEvent) -> crate::message::Msg {
    let modifiers = KeyMod::from(ev.modifiers);
    let mk = |button| Mouse {
        x: ev.column,
        y: ev.row,
        button,
        modifiers,
    };
    match ev.kind {
        MouseEventKind::Down(b) => crate::message::msg(MouseClickMsg(mk(button_from(b)))),
        MouseEventKind::Up(b) => crate::message::msg(MouseReleaseMsg(mk(button_from(b)))),
        MouseEventKind::Drag(b) => crate::message::msg(MouseMotionMsg(mk(button_from(b)))),
        MouseEventKind::Moved => crate::message::msg(MouseMotionMsg(mk(MouseButton::None))),
        MouseEventKind::ScrollUp => crate::message::msg(MouseWheelMsg(mk(MouseButton::WheelUp))),
        MouseEventKind::ScrollDown => {
            crate::message::msg(MouseWheelMsg(mk(MouseButton::WheelDown)))
        }
        MouseEventKind::ScrollLeft => {
            crate::message::msg(MouseWheelMsg(mk(MouseButton::WheelLeft)))
        }
        MouseEventKind::ScrollRight => {
            crate::message::msg(MouseWheelMsg(mk(MouseButton::WheelRight)))
        }
    }
}
