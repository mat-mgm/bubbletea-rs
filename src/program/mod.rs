//! The [`Program`] runtime. Builder lives here; the async event loop is in
//! [`run`]. Port of `tea.go`'s `Program`.

mod run;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::color::ColorProfile;
use crate::error::Result;
use crate::message::Msg;
use crate::model::Model;
use crate::options::Options;

/// A handle to send messages into a running [`Program`] from the outside.
///
/// Analogue of the methods `Program.Send`/`Quit` in Go. Cloneable and `Send`.
#[derive(Clone)]
pub struct Sender {
    msgs: mpsc::UnboundedSender<Msg>,
}

impl Sender {
    /// Inject a message into the program's update loop.
    ///
    /// No-op if the program has already exited.
    pub fn send(&self, msg: Msg) {
        let _ = self.msgs.send(msg);
    }

    /// Convenience: tell the program to quit.
    pub fn quit(&self) {
        self.send(crate::message::msg(crate::message::QuitMsg));
    }
}

/// A terminal user interface program.
///
/// Analogue of Go's `tea.Program`. Build with [`Program::new`], configure with
/// the builder methods, then drive it with [`Program::run`].
pub struct Program<M: Model> {
    pub(crate) model: M,
    pub(crate) opts: Options,
    pub(crate) msg_tx: mpsc::UnboundedSender<Msg>,
    pub(crate) msg_rx: mpsc::UnboundedReceiver<Msg>,
    pub(crate) cancel: CancellationToken,
}

impl<M: Model> Program<M> {
    /// Create a new program with the given initial model.
    pub fn new(model: M) -> Self {
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        Program {
            model,
            opts: Options::default(),
            msg_tx,
            msg_rx,
            cancel: CancellationToken::new(),
        }
    }

    /// Get a [`Sender`] for injecting messages from outside the loop.
    pub fn sender(&self) -> Sender {
        Sender {
            msgs: self.msg_tx.clone(),
        }
    }

    /// A clone of the program's cancellation token. Cancelling it stops the
    /// program (the idiomatic replacement for Go's `WithContext`).
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    // --- Builder options (port of `WithX`) ------------------------------------

    /// Disable all input. Go `WithInput(nil)`.
    pub fn without_input(mut self) -> Self {
        self.opts.disable_input = true;
        self
    }

    /// Handle OS signals yourself. Go `WithoutSignalHandler`.
    pub fn without_signal_handler(mut self) -> Self {
        self.opts.disable_signal_handler = true;
        self
    }

    /// Disable panic catching. Go `WithoutCatchPanics`.
    pub fn without_catch_panics(mut self) -> Self {
        self.opts.disable_catch_panics = true;
        self
    }

    /// Ignore OS signals entirely. Go `WithoutSignals`.
    pub fn without_signals(mut self) -> Self {
        self.opts.ignore_signals = true;
        self
    }

    /// Disable the renderer; output is sent plainly. Go `WithoutRenderer`.
    pub fn without_renderer(mut self) -> Self {
        self.opts.disable_renderer = true;
        self
    }

    /// Set the maximum renderer FPS (clamped to 1..=120). Go `WithFPS`.
    pub fn with_fps(mut self, fps: u16) -> Self {
        self.opts.fps = fps.clamp(1, super::MAX_FPS);
        self
    }

    /// Force a color profile. Go `WithColorProfile`.
    pub fn with_color_profile(mut self, profile: ColorProfile) -> Self {
        self.opts.color_profile = Some(profile);
        self
    }

    /// Set the initial window size. Go `WithWindowSize`.
    pub fn with_window_size(mut self, width: u16, height: u16) -> Self {
        self.opts.window_size = Some((width, height));
        self
    }

    /// Run the program, blocking (on the async runtime) until it exits.
    ///
    /// Analogue of Go's `Program.Run`.
    pub async fn run(self) -> Result<M> {
        run::run(self).await
    }
}
