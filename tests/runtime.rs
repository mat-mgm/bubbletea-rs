//! Integration tests for the core runtime (Phase 1).

use std::time::Duration;

use bubbletea_rs as tea;

struct Inc;
struct Done;

#[derive(Default)]
struct Counter {
    n: i32,
    batched: bool,
}

impl tea::Model for Counter {
    fn init(&mut self) -> Option<tea::Cmd> {
        // Batch two increments, then a quit after a short tick.
        tea::batch(vec![
            Some(tea::cmd(async { Some(tea::msg(Inc)) })),
            Some(tea::cmd(async { Some(tea::msg(Inc)) })),
            Some(tea::tick(Duration::from_millis(20), || Done)),
        ])
    }

    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if msg.is::<Inc>() {
            self.n += 1;
            self.batched = true;
            return None;
        }
        if msg.is::<Done>() {
            return Some(tea::quit());
        }
        None
    }

    fn view(&self) -> tea::View {
        tea::View::new(format!("n={}", self.n))
    }
}

#[tokio::test]
async fn batch_then_quit_returns_final_model() {
    let model = tea::Program::new(Counter::default())
        .without_input()
        .without_renderer()
        .run()
        .await
        .expect("program should exit cleanly");
    assert_eq!(model.n, 2, "both batched increments should apply");
    assert!(model.batched);
}

struct SeqModel {
    order: Vec<u8>,
}

impl tea::Model for SeqModel {
    fn init(&mut self) -> Option<tea::Cmd> {
        tea::sequence(vec![
            Some(tea::cmd(async { Some(tea::msg(1u8)) })),
            Some(tea::cmd(async { Some(tea::msg(2u8)) })),
            Some(tea::cmd(async { Some(tea::msg(3u8)) })),
        ])
    }

    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(&v) = msg.downcast_ref::<u8>() {
            self.order.push(v);
            if self.order.len() == 3 {
                return Some(tea::quit());
            }
        }
        None
    }

    fn view(&self) -> tea::View {
        tea::View::default()
    }
}

#[tokio::test]
async fn sequence_runs_in_order() {
    let model = tea::Program::new(SeqModel { order: vec![] })
        .without_input()
        .without_renderer()
        .run()
        .await
        .unwrap();
    assert_eq!(model.order, vec![1, 2, 3]);
}

struct ExternalQuit {
    ticks: u32,
}

impl tea::Model for ExternalQuit {
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if msg.is::<tea::WindowSizeMsg>() {
            self.ticks += 1;
        }
        None
    }
    fn view(&self) -> tea::View {
        tea::View::default()
    }
}

#[tokio::test]
async fn external_sender_can_quit() {
    let program = tea::Program::new(ExternalQuit { ticks: 0 })
        .without_input()
        .without_renderer();
    let sender = program.sender();
    let handle = tokio::spawn(async move { program.run().await });
    // Give the loop a moment, then quit from outside.
    tokio::time::sleep(Duration::from_millis(20)).await;
    sender.quit();
    let model = handle.await.unwrap().unwrap();
    assert!(model.ticks >= 1, "should have received the initial WindowSizeMsg");
}
