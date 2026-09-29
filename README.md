# bubbletea-rs

A native Rust port of [Bubble Tea](https://charm.land/bubbletea), the terminal UI framework based on The Elm Architecture.

## Features

- The Elm Architecture: clean separation between Model, Update, and View.
- Async runtime powered by Tokio for non-blocking command execution.
- Cross-platform terminal handling via Crossterm.
- Support for inline rendering and full-screen alternative screen mode.
- Event handling for keyboard, mouse (clicks, scrolls, motion), window resize, focus, and paste.
- Terminal capability and color profile detection.
- Subprocess execution with terminal release and restore.
- Clean terminal state management and RAII-based cleanup on exit.

## Installation

Add `bubbletea-rs` to your `Cargo.toml`:

```toml
[dependencies]
bubbletea-rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
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
        if let Some(k) = msg.downcast_ref::<tea::KeyPressMsg>() {
            match k.to_string().as_str() {
                "ctrl+c" | "q" => return Some(tea::quit()),
                _ => {}
            }
        }
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
```

## Core Architecture

Bubble Tea programs consist of three core elements:

1. **Model**: The state of your application.
2. **Update**: A function that handles incoming messages (events, timers, I/O results) and updates state, optionally returning commands.
3. **View**: A function that renders the current state into an ANSI-compatible string representation.

Messages (`tea::Msg`) are dynamically typed and can be matched using `msg.downcast_ref::<T>()` or `msg.is::<T>()`. Commands (`tea::Cmd`) are asynchronous futures that yield optional messages back into the event loop.

## Examples

The repository includes several runnable examples demonstrating different features:

- `simple`: Basic countdown timer demonstrating commands and ticks.
- `altscreen`: Switching between standard output and full-screen alternate buffer.
- `spinner`: Animated terminal spinner.
- `mouse`: Handling mouse click, drag, and scroll events.
- `progress`: Animated progress bar updates.
- `exec`: Spawning external processes and restoring terminal state.
- `send_msg`: Sending messages to a running program from an external thread.
- `print_key`: Inspecting and debugging keyboard events.

Run any example with Cargo:

```bash
cargo run --example simple
```

## License

This project is licensed under the MIT License.
