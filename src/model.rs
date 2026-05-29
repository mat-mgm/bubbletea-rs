//! The [`Model`] trait. Port of Go's `tea.Model` interface.

use crate::command::Cmd;
use crate::message::Msg;
use crate::view::View;

/// A model holds the program's state and its core functions.
///
/// Analogue of Go's `tea.Model`. Note the deliberate divergence: Go's
/// `Update(Msg) (Model, Cmd)` returns a new model by value; here `update` takes
/// `&mut self` and returns just an optional command, mutating state in place to
/// avoid cloning the model on every message.
pub trait Model {
    /// First function called. Returns an optional initial command.
    fn init(&mut self) -> Option<Cmd> {
        None
    }

    /// Called when a message is received. Inspect the message, update state, and
    /// optionally return a command.
    fn update(&mut self, msg: Msg) -> Option<Cmd>;

    /// Render the program's UI. Called after every `update`.
    fn view(&self) -> View;
}
