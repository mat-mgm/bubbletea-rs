//! # bubbletea-rs
//!
//! A Rust port of [Bubble Tea](https://charm.land/bubbletea), a framework for
//! building terminal user interfaces based on The Elm Architecture
//! (Model / Update / View).
//!
//! ```no_run
//! use bubbletea_rs as tea;
//!
//! struct App { n: i32 }
//!
//! impl tea::Model for App {
//!     fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
//!         if msg.is::<tea::QuitMsg>() { return None; }
//!         None
//!     }
//!     fn view(&self) -> tea::View { tea::View::new(format!("n = {}", self.n)) }
//! }
//!
//! # async fn run() -> tea::Result<()> {
//! tea::Program::new(App { n: 0 }).run().await?;
//! # Ok(())
//! # }
//! ```

/// Default renderer frame rate.
pub const DEFAULT_FPS: u16 = 60;
/// Maximum renderer frame rate.
pub const MAX_FPS: u16 = 120;

pub mod color;
pub mod command;
pub mod cursor;
pub mod error;
pub mod focus;
pub mod input;
pub mod key;
pub mod message;
pub mod model;
pub mod mouse;
pub mod options;
pub mod program;
pub mod renderer;
pub mod terminal;
pub mod view;

// --- Public prelude-style re-exports (mirrors the flat `tea.X` Go API) --------

pub use color::{ColorProfile, ColorProfileMsg};
pub use command::{batch, cmd, interrupt, quit, sequence, suspend, tick, Cmd};
pub use cursor::{Cursor, CursorPositionMsg, CursorShape, Position};
pub use error::{Error, Result};
pub use focus::{BlurMsg, FocusMsg, PasteMsg};
pub use key::{Key, KeyCode, KeyMod, KeyPressMsg, KeyReleaseMsg};
pub use message::{
    msg, InterruptMsg, Msg, QuitMsg, ResumeMsg, SuspendMsg, WindowSizeMsg,
};
pub use model::Model;
pub use mouse::{
    Mouse, MouseButton, MouseClickMsg, MouseMotionMsg, MouseReleaseMsg, MouseWheelMsg,
};
pub use program::{Program, Sender};
pub use view::{MouseMode, View};
