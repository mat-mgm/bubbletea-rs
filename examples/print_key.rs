//! Port of bubbletea's `examples/print-key`: echoes key, mouse, and focus events.

use bubbletea_rs as tea;

#[derive(Default)]
struct Model {
    last: String,
}

impl tea::Model for Model {
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(k) = msg.downcast_ref::<tea::KeyPressMsg>() {
            if k.to_string() == "ctrl+c" || k.to_string() == "q" {
                return Some(tea::quit());
            }
            self.last = format!("key: {k}");
        } else if let Some(m) = msg.downcast_ref::<tea::MouseClickMsg>() {
            self.last = format!("mouse click: {} at {},{}", m.0, m.0.x, m.0.y);
        } else if msg.is::<tea::FocusMsg>() {
            self.last = "focus gained".into();
        } else if msg.is::<tea::BlurMsg>() {
            self.last = "focus lost".into();
        } else if let Some(p) = msg.downcast_ref::<tea::PasteMsg>() {
            self.last = format!("paste: {}", p.0);
        }
        None
    }

    fn view(&self) -> tea::View {
        let mut v = tea::View::new(format!(
            "Press keys (q or ctrl+c to quit).\nLast event: {}\n",
            self.last
        ));
        v.mouse_mode = tea::MouseMode::CellMotion;
        v.report_focus = true;
        v
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(Model::default()).run().await?;
    Ok(())
}
