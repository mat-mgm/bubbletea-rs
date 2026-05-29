//! Commands.
//!
//! Port of Go's `Cmd func() Msg`. A command is an async operation that resolves
//! to an optional message. Returning `None` is the analogue of returning a `nil`
//! message in Go (a no-op).

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::message::{BatchMsg, Msg, QuitMsg, SequenceMsg, SuspendMsg};

/// A command is an IO operation that returns a message when complete.
///
/// Analogue of Go's `tea.Cmd`. Use it for things like HTTP requests, timers,
/// reading and writing files, and so on.
pub type Cmd = Pin<Box<dyn Future<Output = Option<Msg>> + Send>>;

/// Build a [`Cmd`] from any future resolving to an optional message.
pub fn cmd<F>(future: F) -> Cmd
where
    F: Future<Output = Option<Msg>> + Send + 'static,
{
    Box::pin(future)
}

/// A command that tells the program to quit.
pub fn quit() -> Cmd {
    cmd(async { Some(crate::message::msg(QuitMsg)) })
}

/// A command that tells the program to suspend (as with ctrl+z).
pub fn suspend() -> Cmd {
    cmd(async { Some(crate::message::msg(SuspendMsg)) })
}

/// A command that tells the program to interrupt (as with ctrl+c).
pub fn interrupt() -> Cmd {
    cmd(async { Some(crate::message::msg(crate::message::InterruptMsg)) })
}

/// Drop any `None` commands and collapse to the most direct form, mirroring Go's
/// `compactCmds`: zero → `None`, one → that command, many → `Some(vec)`.
fn compact(cmds: Vec<Option<Cmd>>) -> Option<Vec<Cmd>> {
    let valid: Vec<Cmd> = cmds.into_iter().flatten().collect();
    if valid.is_empty() {
        None
    } else {
        Some(valid)
    }
}

/// Run a bunch of commands concurrently with no ordering guarantees.
///
/// Analogue of `tea.Batch`.
pub fn batch(cmds: Vec<Option<Cmd>>) -> Option<Cmd> {
    let valid = compact(cmds)?;
    if valid.len() == 1 {
        return valid.into_iter().next();
    }
    Some(cmd(async move { Some(crate::message::msg(BatchMsg(valid))) }))
}

/// Run the given commands one at a time, in order.
///
/// Analogue of `tea.Sequence`.
pub fn sequence(cmds: Vec<Option<Cmd>>) -> Option<Cmd> {
    let valid = compact(cmds)?;
    if valid.len() == 1 {
        return valid.into_iter().next();
    }
    Some(cmd(async move { Some(crate::message::msg(SequenceMsg(valid))) }))
}

/// A command that produces a message after a delay, then stops.
///
/// Analogue of `tea.Tick`. `f` maps the elapsed instant to a message.
pub fn tick<F, T>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    cmd(async move {
        tokio::time::sleep(duration).await;
        Some(crate::message::msg(f()))
    })
}

/// A command that ticks in sync with the system clock.
///
/// Analogue of `tea.Every`. Unlike [`tick`], which measures duration from
/// invocation, `every` aligns to clock-truncated boundaries. Pass a duration
/// and a mapping function that turns a `std::time::Instant` into a message.
pub fn every<F, T>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce(std::time::Instant) -> T + Send + 'static,
    T: Send + 'static,
{
    cmd(async move {
        // Compute delay to the next aligned boundary.
        let now = tokio::time::Instant::now();
        let elapsed_nanos = now.elapsed().as_nanos();
        let dur_nanos = duration.as_nanos().max(1);
        let remaining = dur_nanos - (elapsed_nanos % dur_nanos);
        tokio::time::sleep(Duration::from_nanos(remaining as u64)).await;
        Some(crate::message::msg(f(std::time::Instant::now())))
    })
}

/// Print a line above the program (above the managed TUI frame).
///
/// Analogue of `p.Println`. Unlike direct `println!`, output from this command
/// is correctly interleaved with the renderer's cursor management.
pub fn println(s: impl Into<String> + Send + 'static) -> Cmd {
    cmd(async move { Some(crate::message::msg(crate::message::PrintLineMsg(s.into()))) })
}

/// Print a formatted line above the program.
///
/// Analogue of `p.Printf`.
pub fn printf(template: impl std::fmt::Display + Send + 'static) -> Cmd {
    cmd(async move {
        Some(crate::message::msg(crate::message::PrintLineMsg(
            template.to_string(),
        )))
    })
}

/// Request the current terminal window size.
///
/// Analogue of `tea.RequestWindowSize`. A [`crate::WindowSizeMsg`] is delivered
/// to `update`. This is rarely needed because size is sent automatically on
/// start and on resize.
pub fn request_window_size() -> Cmd {
    cmd(async move { Some(crate::message::msg(crate::message::RequestWindowSizeMsg)) })
}

/// Send a raw ANSI/VT sequence directly to the terminal output.
///
/// Analogue of `tea.Raw`.
pub fn raw(seq: impl Into<String> + Send + 'static) -> Cmd {
    cmd(async move { Some(crate::message::msg(crate::message::RawMsg(seq.into()))) })
}
