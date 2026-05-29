//! Progress bar demo using OSC 9;4 (Windows Terminal) and a text bar.
//! Press space to advance, 'r' to reset, 'q' to quit.

use bubbletea_rs as tea;

struct App {
    value: u8,
}

impl tea::Model for App {
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(key) = msg.downcast_ref::<tea::KeyPressMsg>() {
            match key.0.code {
                tea::KeyCode::Char('q') => return Some(tea::quit()),
                tea::KeyCode::Char('r') => self.value = 0,
                tea::KeyCode::Char(' ') => {
                    self.value = self.value.saturating_add(10).min(100);
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> tea::View {
        let v = self.value;
        let filled = (v as usize * 40 / 100).min(40);
        let bar = format!("[{}{}] {}%", "█".repeat(filled), "░".repeat(40 - filled), v);
        let content = format!("Progress bar demo\n{bar}\n[space] +10  [r] reset  [q] quit\n");

        let state = if v >= 100 {
            tea::ProgressBarState::Default
        } else {
            tea::ProgressBarState::Default
        };
        let mut view = tea::View::new(content);
        view.progress_bar = Some(tea::ProgressBar::new(state, v));
        view
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(App { value: 0 }).run().await?;
    Ok(())
}
