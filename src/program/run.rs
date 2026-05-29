//! The async event loop. Port of `tea.go`'s `Run`/`eventLoop`/`handleCommands`.

use std::panic::AssertUnwindSafe;

use futures::FutureExt;
use tokio::sync::mpsc;

use std::time::Duration;

use super::Program;
use crate::command::Cmd;
use crate::error::{Error, Result};
use crate::clipboard::{
    ReadClipboardMsg, ReadPrimaryClipboardMsg, SetClipboardMsg, SetPrimaryClipboardMsg,
};
use crate::message::{
    BatchMsg, ClearScreenMsg, InterruptMsg, Msg, PrintLineMsg, QuitMsg, RawMsg,
    RequestWindowSizeMsg, SequenceMsg, SuspendMsg, WindowSizeMsg,
};
use crate::model::Model;
use crate::renderer::{NilRenderer, Renderer, StandardRenderer};
use crate::terminal::TerminalGuard;

/// Outcome of handling a single message in the loop.
enum Flow {
    Continue,
    Quit,
    Interrupted,
}

pub(crate) async fn run<M: Model>(mut program: Program<M>) -> Result<M> {
    let msg_tx = program.msg_tx.clone();

    // Terminal setup (raw mode + RAII restore). Skipped when input is disabled.
    let mut guard = if program.opts.disable_input {
        None
    } else {
        Some(TerminalGuard::new()?)
    };

    // Spawn the input reader when attached to a real terminal.
    if let Some(g) = guard.as_ref() {
        if g.is_tty() {
            crate::input::spawn(msg_tx.clone(), program.cancel.clone());
        }
    }

    // Select a renderer.
    let mut renderer: Box<dyn Renderer> = if program.opts.disable_renderer {
        Box::new(NilRenderer)
    } else {
        Box::new(StandardRenderer::new(80, 24))
    };

    // Determine and seed the initial window size: explicit override, else the
    // real terminal size, else the 80x24 fallback.
    let (w, h) = program.opts.window_size.unwrap_or_else(|| {
        guard.as_ref().map(|g| g.size()).unwrap_or((80, 24))
    });
    renderer.resize(w, h);
    let _ = msg_tx.send(crate::message::msg(WindowSizeMsg {
        width: w,
        height: h,
    }));

    // Run the initial command, if any.
    if let Some(cmd) = program.model.init() {
        dispatch(cmd, &msg_tx, program.opts.disable_catch_panics);
    }

    // Initial render.
    apply_view(&program, &mut renderer, guard.as_mut());

    // Renderer flush ticker (port of the fps ticker in `startRenderer`).
    let framerate = Duration::from_secs(1) / program.opts.fps.max(1) as u32;
    let mut ticker = tokio::time::interval(framerate);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let result = loop {
        tokio::select! {
            _ = program.cancel.cancelled() => break Ok(()),
            _ = ticker.tick() => {
                renderer.flush(false);
            }
            maybe_msg = program.msg_rx.recv() => {
                let Some(msg) = maybe_msg else { break Ok(()); };
                match handle(&mut program, msg, &msg_tx, &mut renderer, guard.as_mut()) {
                    Flow::Continue => {}
                    Flow::Quit => break Ok(()),
                    Flow::Interrupted => break Err(Error::Interrupted),
                }
            }
        }
    };

    renderer.flush(true);
    renderer.close();
    if let Some(mut g) = guard {
        g.restore();
    }
    result.map(|()| program.model)
}

/// Render the current view and reconcile terminal state (alt screen, mouse,
/// focus reporting, bracketed paste) with it.
fn apply_view<M: Model>(
    program: &Program<M>,
    renderer: &mut Box<dyn Renderer>,
    guard: Option<&mut TerminalGuard>,
) {
    let view = program.model.view();
    if let Some(g) = guard {
        let _ = g.set_alt_screen(view.alt_screen);
        let _ = g.set_mouse(view.mouse_mode);
        let _ = g.set_focus(view.report_focus);
        let _ = g.set_bracketed_paste(!view.disable_bracketed_paste);
    }
    renderer.render(&view);
}

/// Handle one message: process internal control messages, then run `update` and
/// dispatch the resulting command, then render.
fn handle<M: Model>(
    program: &mut Program<M>,
    msg: Msg,
    msg_tx: &mpsc::UnboundedSender<Msg>,
    renderer: &mut Box<dyn Renderer>,
    guard: Option<&mut TerminalGuard>,
) -> Flow {
    let catch = program.opts.disable_catch_panics;

    // Internal control messages handled by the runtime before reaching update.
    // Each `downcast` threads ownership: `Ok` consumes the box, `Err` returns it.
    let msg = match msg.downcast::<QuitMsg>() {
        Ok(_) => return Flow::Quit,
        Err(m) => m,
    };
    let msg = match msg.downcast::<InterruptMsg>() {
        Ok(_) => return Flow::Interrupted,
        Err(m) => m,
    };
    let msg = match msg.downcast::<SuspendMsg>() {
        // Suspend support is implemented in Phase 6.
        Ok(_) => return Flow::Continue,
        Err(m) => m,
    };
    let msg = match msg.downcast::<BatchMsg>() {
        Ok(batch) => {
            for cmd in batch.0 {
                dispatch(cmd, msg_tx, catch);
            }
            return Flow::Continue;
        }
        Err(m) => m,
    };
    let msg = match msg.downcast::<SequenceMsg>() {
        Ok(seq) => {
            dispatch_sequence(seq.0, msg_tx, catch);
            return Flow::Continue;
        }
        Err(m) => m,
    };
    let msg = match msg.downcast::<PrintLineMsg>() {
        Ok(line) => {
            renderer.insert_above(&line.0);
            return Flow::Continue;
        }
        Err(m) => m,
    };
    let msg = match msg.downcast::<ClearScreenMsg>() {
        Ok(_) => {
            renderer.clear_screen();
            return Flow::Continue;
        }
        Err(m) => m,
    };

    // Raw ANSI sequence: write directly to stdout.
    let msg = match msg.downcast::<RawMsg>() {
        Ok(raw) => {
            use std::io::Write;
            let _ = std::io::stdout().write_all(raw.0.as_bytes());
            return Flow::Continue;
        }
        Err(m) => m,
    };

    // Clipboard write/read commands via OSC 52.
    let msg = match msg.downcast::<SetClipboardMsg>() {
        Ok(m) => {
            use std::io::Write;
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(m.0.as_bytes());
            let _ = write!(std::io::stdout(), "\x1b]52;c;{encoded}\x07");
            return Flow::Continue;
        }
        Err(m) => m,
    };
    let msg = match msg.downcast::<SetPrimaryClipboardMsg>() {
        Ok(m) => {
            use std::io::Write;
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(m.0.as_bytes());
            let _ = write!(std::io::stdout(), "\x1b]52;p;{encoded}\x07");
            return Flow::Continue;
        }
        Err(m) => m,
    };
    let msg = match msg.downcast::<ReadClipboardMsg>() {
        Ok(_) => {
            use std::io::Write;
            let _ = write!(std::io::stdout(), "\x1b]52;c;?\x07");
            return Flow::Continue;
        }
        Err(m) => m,
    };
    let msg = match msg.downcast::<ReadPrimaryClipboardMsg>() {
        Ok(_) => {
            use std::io::Write;
            let _ = write!(std::io::stdout(), "\x1b]52;p;?\x07");
            return Flow::Continue;
        }
        Err(m) => m,
    };
    // RequestWindowSizeMsg: trigger a fresh size check via a WindowSizeMsg.
    let msg = match msg.downcast::<RequestWindowSizeMsg>() {
        Ok(_) => {
            if let Some(g) = guard.as_ref() {
                let (w, h) = g.size();
                let _ = msg_tx.send(crate::message::msg(WindowSizeMsg {
                    width: w,
                    height: h,
                }));
            }
            return Flow::Continue;
        }
        Err(m) => m,
    };

    // WindowSizeMsg is handled by the renderer AND delivered to update (Go behavior).
    if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
        renderer.resize(size.width, size.height);
    }

    // Deliver to the model.
    if let Some(cmd) = program.model.update(msg) {
        dispatch(cmd, msg_tx, catch);
    }

    apply_view(program, renderer, guard);
    Flow::Continue
}

/// Run a command as a tokio task, sending its resulting message back to the loop.
/// Port of `handleCommands`' per-command goroutine.
fn dispatch(cmd: Cmd, msg_tx: &mpsc::UnboundedSender<Msg>, catch_panics: bool) {
    let tx = msg_tx.clone();
    tokio::spawn(async move {
        let result = if catch_panics {
            match AssertUnwindSafe(cmd).catch_unwind().await {
                Ok(msg) => msg,
                Err(_) => Some(crate::message::msg(QuitMsg)),
            }
        } else {
            cmd.await
        };
        if let Some(msg) = result {
            let _ = tx.send(msg);
        }
    });
}

/// Run commands one at a time, in order. Port of `execSequenceMsg`.
fn dispatch_sequence(cmds: Vec<Cmd>, msg_tx: &mpsc::UnboundedSender<Msg>, catch_panics: bool) {
    let tx = msg_tx.clone();
    tokio::spawn(async move {
        for cmd in cmds {
            let result = if catch_panics {
                (AssertUnwindSafe(cmd).catch_unwind().await).unwrap_or(None)
            } else {
                cmd.await
            };
            if let Some(msg) = result {
                // Recursively handle nested batch/sequence by re-injecting.
                let _ = tx.send(msg);
            }
        }
    });
}
