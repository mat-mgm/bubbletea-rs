//! Port of bubbletea's `examples/simple`: counts down from 5 and exits.

use std::time::Duration;

use bubbletea_rs as tea;

struct Model {
    count: i32,
}

struct TickMsg;

fn tick() -> tea::Cmd {
    tea::tick(Duration::from_secs(1), || TickMsg)
}

impl tea::Model for Model {
    fn init(&mut self) -> Option<tea::Cmd> {
        Some(tick())
    }

    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if msg.is::<TickMsg>() {
            self.count -= 1;
            if self.count <= 0 {
                return Some(tea::quit());
            }
            return Some(tick());
        }
        None
    }

    fn view(&self) -> tea::View {
        tea::View::new(format!(
            "Hi. This program will exit in {} seconds.\n",
            self.count
        ))
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(Model { count: 5 }).run().await?;
    Ok(())
}
