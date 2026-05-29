//! Animated spinner using tick commands. Press 'q' to quit.

use std::time::Duration;
use bubbletea_rs as tea;

const FRAMES: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

struct App {
    frame: usize,
    ticking: bool,
}

#[derive(Debug)]
struct Tick;

impl tea::Model for App {
    fn init(&mut self) -> Option<tea::Cmd> {
        self.ticking = true;
        Some(next_tick())
    }

    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(key) = msg.downcast_ref::<tea::KeyPressMsg>() {
            if matches!(key.0.code, tea::KeyCode::Char('q')) {
                return Some(tea::quit());
            }
        }
        if msg.downcast_ref::<Tick>().is_some() {
            self.frame = (self.frame + 1) % FRAMES.len();
            return Some(next_tick());
        }
        None
    }

    fn view(&self) -> tea::View {
        tea::View::new(format!("{} Loading...  press q to quit\n", FRAMES[self.frame]))
    }
}

fn next_tick() -> tea::Cmd {
    tea::tick(Duration::from_millis(100), || Tick)
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(App { frame: 0, ticking: false }).run().await?;
    Ok(())
}
