//! Demonstrates exec_process: launch an external editor, then return to the TUI.
//!
//! Press 'e' to open $EDITOR (falls back to `vi`), 'q' to quit.

use std::process::Command;
use bubbletea_rs as tea;

struct App {
    status: String,
}

#[derive(Debug)]
struct EditorDone(Option<tea::Error>);

impl tea::Model for App {
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(key) = msg.downcast_ref::<tea::KeyPressMsg>() {
            match key.0.code {
                tea::KeyCode::Char('q') => return Some(tea::quit()),
                tea::KeyCode::Char('e') => {
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                    return Some(tea::exec_process(
                        Command::new(editor),
                        |err| Some(tea::msg(EditorDone(err))),
                    ));
                }
                _ => {}
            }
        }
        if let Some(done) = msg.downcast_ref::<EditorDone>() {
            self.status = match &done.0 {
                None => "Editor closed successfully.".to_string(),
                Some(e) => format!("Editor error: {e}"),
            };
        }
        None
    }

    fn view(&self) -> tea::View {
        tea::View::new(format!(
            "Press 'e' to open editor, 'q' to quit.\n{}\n",
            self.status
        ))
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(App { status: String::new() }).run().await?;
    Ok(())
}
