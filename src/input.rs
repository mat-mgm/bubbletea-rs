//! Input reader. Port of the input loop in `tea.go` (`initInputReader`).
//!
//! Reads terminal events from crossterm's async [`EventStream`] and translates
//! them into [`Msg`](crate::message::Msg)s on the program's channel. Runs as a
//! tokio task and stops when the program's cancellation token fires.

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::focus::{BlurMsg, FocusMsg, PasteMsg};
use crate::message::{msg, Msg, WindowSizeMsg};
use crate::{key, mouse};

/// Spawn the input reader. Translates crossterm events to messages until the
/// stream ends or `cancel` is triggered.
pub(crate) fn spawn(msg_tx: mpsc::UnboundedSender<Msg>, cancel: CancellationToken) {
    tokio::spawn(async move {
        let mut events = EventStream::new();
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                maybe = events.next() => {
                    let Some(Ok(event)) = maybe else { break };
                    if let Some(m) = translate(event) {
                        if msg_tx.send(m).is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });
}

/// Translate a single crossterm [`Event`] into a Bubble Tea message.
fn translate(event: Event) -> Option<Msg> {
    match event {
        Event::Key(ev) => key::from_crossterm(ev),
        Event::Mouse(ev) => Some(mouse::from_crossterm(ev)),
        Event::Resize(w, h) => Some(msg(WindowSizeMsg {
            width: w,
            height: h,
        })),
        Event::FocusGained => Some(msg(FocusMsg)),
        Event::FocusLost => Some(msg(BlurMsg)),
        Event::Paste(content) => Some(msg(PasteMsg(content))),
    }
}
