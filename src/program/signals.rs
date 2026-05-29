//! OS signal handling. Port of `signals.go`.
//!
//! Listens for SIGINT/SIGTERM (quit), SIGWINCH (resize), and SIGTSTP (suspend)
//! and translates them into Bubble Tea messages. Windows stubs are provided via
//! `cfg` guards.

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::message::{msg, InterruptMsg, Msg};

/// Spawn signal listener tasks. Each signal is translated to a `Msg` and sent
/// to the program's message channel.
///
/// On non-Unix platforms only SIGINT/SIGTERM equivalents are wired; SIGWINCH
/// and SIGTSTP are not available.
pub(crate) fn spawn(
    msg_tx: mpsc::UnboundedSender<Msg>,
    cancel: CancellationToken,
) {
    #[cfg(unix)]
    spawn_unix(msg_tx, cancel);

    #[cfg(not(unix))]
    spawn_ctrlc(msg_tx, cancel);
}

#[cfg(not(unix))]
fn spawn_ctrlc(msg_tx: mpsc::UnboundedSender<Msg>, cancel: CancellationToken) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = tokio::signal::ctrl_c() => {
                    let _ = msg_tx.send(msg(InterruptMsg));
                }
            }
        }
    });
}

#[cfg(unix)]
fn spawn_unix(msg_tx: mpsc::UnboundedSender<Msg>, cancel: CancellationToken) {
    use tokio::signal::unix::{signal, SignalKind};

    let tx_int = msg_tx.clone();
    let cancel_int = cancel.clone();
    tokio::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT listener");
        let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM listener");
        loop {
            tokio::select! {
                _ = cancel_int.cancelled() => break,
                _ = sigint.recv() => {
                    let _ = tx_int.send(msg(InterruptMsg));
                }
                _ = sigterm.recv() => {
                    let _ = tx_int.send(msg(InterruptMsg));
                }
            }
        }
    });

    // SIGWINCH: terminal resize.
    let tx_winch = msg_tx.clone();
    let cancel_winch = cancel.clone();
    tokio::spawn(async move {
        let mut sigwinch = signal(SignalKind::window_change()).expect("SIGWINCH listener");
        loop {
            tokio::select! {
                _ = cancel_winch.cancelled() => break,
                _ = sigwinch.recv() => {
                    // Query actual size; fall back to 80x24.
                    let (w, h) = crossterm::terminal::size().unwrap_or((80, 24));
                    let _ = tx_winch.send(msg(crate::message::WindowSizeMsg { width: w, height: h }));
                }
            }
        }
    });

    // SIGTSTP: suspend (ctrl+z).
    let tx_stop = msg_tx;
    let cancel_stop = cancel;
    tokio::spawn(async move {
        let mut sigtstp = signal(SignalKind::from_raw(libc::SIGTSTP)).expect("SIGTSTP listener");
        loop {
            tokio::select! {
                _ = cancel_stop.cancelled() => break,
                _ = sigtstp.recv() => {
                    let _ = tx_stop.send(msg(crate::message::SuspendMsg));
                }
            }
        }
    });
}
