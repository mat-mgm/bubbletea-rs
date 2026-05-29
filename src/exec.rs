//! Interactive subprocess execution. Port of `exec.go`.
//!
//! Releases the terminal to a child process (e.g. an editor), then restores it
//! when the subprocess exits. The result is delivered back as a user-defined
//! message via a callback.

use std::process::Command;

use crate::command::{cmd, Cmd};
use crate::error::Error;
use crate::message::{msg, Msg};

/// Callback invoked after the subprocess exits. Maps the exit status to a
/// user message; return `None` to send nothing.
pub type ExecCallback = Box<dyn FnOnce(Option<Error>) -> Option<Msg> + Send + 'static>;

/// Internal message carrying a command to exec. Handled by the runtime before
/// it reaches `Model::update`.
pub(crate) struct ExecMsg {
    pub(crate) command: Command,
    pub(crate) callback: ExecCallback,
}

/// Spawn an interactive subprocess, releasing the terminal for the duration.
///
/// `callback` is called with `None` on success or `Some(err)` on failure and
/// its return value (if any) is re-injected as a message.
///
/// ```no_run
/// use bubbletea_rs::exec_process;
/// use std::process::Command;
///
/// let cmd = exec_process(Command::new("vim"), |err| {
///     if err.is_some() { eprintln!("editor failed"); }
///     None
/// });
/// ```
pub fn exec_process<F>(command: Command, callback: F) -> Cmd
where
    F: FnOnce(Option<Error>) -> Option<Msg> + Send + 'static,
{
    cmd(async move {
        Some(msg(ExecMsg {
            command,
            callback: Box::new(callback),
        }))
    })
}
