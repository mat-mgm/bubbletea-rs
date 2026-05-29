//! Display mouse events. Press 'q' to quit.

use bubbletea_rs as tea;

struct App {
    last: String,
}

impl tea::Model for App {
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(key) = msg.downcast_ref::<tea::KeyPressMsg>() {
            if matches!(key.0.code, tea::KeyCode::Char('q')) {
                return Some(tea::quit());
            }
        }
        if let Some(m) = msg.downcast_ref::<tea::MouseClickMsg>() {
            self.last = format!("click ({}, {}) btn={:?}", m.0.x, m.0.y, m.0.button);
        } else if let Some(m) = msg.downcast_ref::<tea::MouseReleaseMsg>() {
            self.last = format!("release ({}, {})", m.0.x, m.0.y);
        } else if let Some(m) = msg.downcast_ref::<tea::MouseMotionMsg>() {
            self.last = format!("motion ({}, {})", m.0.x, m.0.y);
        } else if let Some(m) = msg.downcast_ref::<tea::MouseWheelMsg>() {
            self.last = format!("wheel ({}, {}) btn={:?}", m.0.x, m.0.y, m.0.button);
        }
        None
    }

    fn view(&self) -> tea::View {
        let mut v = tea::View::new(format!("Mouse events (q to quit)\nLast: {}\n", self.last));
        v.mouse_mode = tea::MouseMode::AllMotion;
        v
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(App { last: "(none)".to_string() }).run().await?;
    Ok(())
}
