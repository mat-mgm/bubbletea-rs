//! Toggle between inline and alt-screen mode.
//! Press 'a' to toggle alt-screen, 'q' / ctrl+c to quit.

use bubbletea_rs as tea;

struct App {
    alt: bool,
    lines: Vec<String>,
}

impl tea::Model for App {
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(key) = msg.downcast_ref::<tea::KeyPressMsg>() {
            match key.0.code {
                tea::KeyCode::Char('q') => return Some(tea::quit()),
                tea::KeyCode::Char('a') => self.alt = !self.alt,
                tea::KeyCode::Enter => self.lines.push(format!("line {}", self.lines.len() + 1)),
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> tea::View {
        let mode = if self.alt { "alt-screen" } else { "inline" };
        let content = format!(
            "Mode: {mode}  [a] toggle  [enter] add line  [q] quit\n{}",
            self.lines.join("\n")
        );
        let mut v = tea::View::new(content);
        v.alt_screen = self.alt;
        v
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(App { alt: false, lines: Vec::new() }).run().await?;
    Ok(())
}
