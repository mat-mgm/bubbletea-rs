//! Demonstrate sending messages from outside the program loop.
//!
//! After 200ms the main task sends an external message that updates the UI.
//! The program then waits for 'q' to quit.

use std::time::Duration;
use bubbletea_rs as tea;

struct App {
    received: bool,
}

#[derive(Debug)]
struct ExternalPing;

impl tea::Model for App {
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if msg.downcast_ref::<ExternalPing>().is_some() {
            self.received = true;
        }
        if let Some(key) = msg.downcast_ref::<tea::KeyPressMsg>() {
            if matches!(key.0.code, tea::KeyCode::Char('q')) {
                return Some(tea::quit());
            }
        }
        None
    }

    fn view(&self) -> tea::View {
        let status = if self.received {
            "External ping received!"
        } else {
            "Waiting for external message..."
        };
        tea::View::new(format!("{status}\nPress q to quit\n"))
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    let program = tea::Program::new(App { received: false });
    let sender = program.sender();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        sender.send(tea::msg(ExternalPing));
    });

    program.run().await?;
    Ok(())
}
