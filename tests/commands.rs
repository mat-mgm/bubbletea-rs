//! Tests for command combinators: tick, every, batch, sequence, raw, println.

use std::time::Duration;
use bubbletea_rs as tea;

// --- tick fires once after the duration ---

#[derive(Default)]
struct TickModel { fired: bool }

impl tea::Model for TickModel {
    fn init(&mut self) -> Option<tea::Cmd> {
        Some(tea::tick(Duration::from_millis(10), || "fired"))
    }
    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if msg.downcast_ref::<&str>().copied() == Some("fired") {
            self.fired = true;
            return Some(tea::quit());
        }
        None
    }
    fn view(&self) -> tea::View { tea::View::default() }
}

#[tokio::test]
async fn tick_fires_once() {
    let m = tea::Program::new(TickModel::default())
        .without_input().without_renderer().run().await.unwrap();
    assert!(m.fired);
}

// --- raw command does not crash ---

struct RawModel;
impl tea::Model for RawModel {
    fn init(&mut self) -> Option<tea::Cmd> {
        tea::batch(vec![Some(tea::raw("\x1b[0m")), Some(tea::quit())])
    }
    fn update(&mut self, _: tea::Msg) -> Option<tea::Cmd> { None }
    fn view(&self) -> tea::View { tea::View::default() }
}

#[tokio::test]
async fn raw_command_does_not_crash() {
    tea::Program::new(RawModel)
        .without_input().without_renderer().run().await.unwrap();
}

// --- quit() from init exits cleanly with final model ---

#[derive(Default)]
struct QuitModel { value: u32 }
impl tea::Model for QuitModel {
    fn init(&mut self) -> Option<tea::Cmd> {
        self.value = 42;
        Some(tea::quit())
    }
    fn update(&mut self, _: tea::Msg) -> Option<tea::Cmd> { None }
    fn view(&self) -> tea::View { tea::View::default() }
}

#[tokio::test]
async fn quit_from_init_returns_model() {
    let m = tea::Program::new(QuitModel::default())
        .without_input().without_renderer().run().await.unwrap();
    assert_eq!(m.value, 42);
}

// --- batch of None is a no-op ---

struct NoBatchModel { done: bool }
impl tea::Model for NoBatchModel {
    fn init(&mut self) -> Option<tea::Cmd> {
        let noop = tea::batch(vec![None, None]);
        assert!(noop.is_none(), "batch of all Nones should be None");
        self.done = true;
        Some(tea::quit())
    }
    fn update(&mut self, _: tea::Msg) -> Option<tea::Cmd> { None }
    fn view(&self) -> tea::View { tea::View::default() }
}

#[tokio::test]
async fn batch_of_nones_is_none() {
    let m = tea::Program::new(NoBatchModel { done: false })
        .without_input().without_renderer().run().await.unwrap();
    assert!(m.done);
}
