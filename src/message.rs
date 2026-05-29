//! Message type and internal control messages.
//!
//! Port of Go's `Msg = any`. In Bubble Tea a message is any value; the model's
//! `update` inspects it with a type switch. The Rust equivalent is a boxed
//! `Any`, inspected with [`downcast_ref`](std::any::Any::downcast_ref) or
//! [`is`](std::any::Any::is).

use std::any::Any;

/// A message carries data from the result of an IO operation. Messages trigger
/// the model's `update` function and, henceforth, the UI.
///
/// This is the analogue of Go's `tea.Msg` (`any`). Match on it via
/// `msg.downcast_ref::<MyMsg>()`.
pub type Msg = Box<dyn Any + Send>;

/// Convenience constructor: box any value into a [`Msg`].
pub fn msg<T: Any + Send>(value: T) -> Msg {
    Box::new(value)
}

// --- Internal control messages -------------------------------------------------
// These mirror Bubble Tea's special messages handled directly by the runtime.

/// Signals that the program should quit. Send it with [`crate::quit`].
#[derive(Debug, Clone, Copy)]
pub struct QuitMsg;

/// Signals the program should suspend (e.g. ctrl+z). Send with [`crate::suspend`].
#[derive(Debug, Clone, Copy)]
pub struct SuspendMsg;

/// Sent once a program resumes from a suspended state.
#[derive(Debug, Clone, Copy)]
pub struct ResumeMsg;

/// Signals the program should interrupt (e.g. ctrl+c). Send with [`crate::interrupt`].
#[derive(Debug, Clone, Copy)]
pub struct InterruptMsg;

/// Reports the terminal window size. Sent on startup and on resize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSizeMsg {
    pub width: u16,
    pub height: u16,
}

/// Internal: a batch of commands to run concurrently.
pub(crate) struct BatchMsg(pub(crate) Vec<crate::command::Cmd>);

/// Internal: a sequence of commands to run in order.
pub(crate) struct SequenceMsg(pub(crate) Vec<crate::command::Cmd>);

/// Internal: print a line above the program (Println/Printf).
pub(crate) struct PrintLineMsg(pub(crate) String);

/// Internal: clear the screen.
pub(crate) struct ClearScreenMsg;
