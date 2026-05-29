# Go → Rust API Mapping

Living table mapping Bubble Tea (Go) symbols to their `bubbletea-rs` equivalents.
Updated as each phase lands. `tea.` is the Go package; `crate::` is this crate.

## Core types

| Go | Rust | Notes |
|---|---|---|
| `tea.Msg` (`any`) | `tea::Msg` = `Box<dyn Any + Send>` | match with `msg.downcast::<T>()` / `msg.is::<T>()` |
| `tea.Cmd` (`func() Msg`) | `tea::Cmd` = `Pin<Box<dyn Future<Output=Option<Msg>> + Send>>` | async; `None` ≈ nil |
| `tea.Model` interface | `tea::Model` trait | `update(&mut self) -> Option<Cmd>` (in-place; diverges from Go's value return) |
| `tea.View` | `tea::View` | core fields in Phase 0; full fields in Phase 5 |
| `tea.Program` | `tea::Program<M>` | builder methods replace `WithX` options |
| `tea.Cursor` / `Position` / `CursorShape` | `tea::Cursor` / `Position` / `CursorShape` | |
| `colorprofile.Profile` | `tea::ColorProfile` | full detection/degrade in Phase 5 |

## Control messages & commands

| Go | Rust |
|---|---|
| `tea.QuitMsg` / `tea.Quit` | `tea::QuitMsg` / `tea::quit()` |
| `tea.InterruptMsg` / `tea.Interrupt` | `tea::InterruptMsg` / `tea::interrupt()` |
| `tea.SuspendMsg` / `tea.Suspend` | `tea::SuspendMsg` / `tea::suspend()` |
| `tea.ResumeMsg` | `tea::ResumeMsg` |
| `tea.WindowSizeMsg` | `tea::WindowSizeMsg` |
| `tea.Batch` | `tea::batch(Vec<Option<Cmd>>)` |
| `tea.Sequence` | `tea::sequence(Vec<Option<Cmd>>)` |
| `tea.Tick` | `tea::tick(Duration, FnOnce)` |
| `p.Send` / `p.Quit` | `Sender::send` / `Sender::quit` |

## Options

| Go | Rust builder |
|---|---|
| `WithInput(nil)` | `.without_input()` |
| `WithoutSignalHandler()` | `.without_signal_handler()` |
| `WithoutCatchPanics()` | `.without_catch_panics()` |
| `WithoutSignals()` | `.without_signals()` |
| `WithoutRenderer()` | `.without_renderer()` |
| `WithFPS(n)` | `.with_fps(n)` |
| `WithColorProfile(p)` | `.with_color_profile(p)` |
| `WithWindowSize(w,h)` | `.with_window_size(w,h)` |
| `WithContext(ctx)` | _TBD_ — cancellation via `CancellationToken` (Phase 1) |
| `WithEnvironment(env)` | _TBD_ (Phase 5) |
| `WithFilter(fn)` | _TBD_ (Phase 1) |

## Go→Rust idiom substitutions

| Go | Rust |
|---|---|
| goroutine | `tokio::spawn` |
| `chan T` | `tokio::sync::mpsc` / `oneshot` |
| `defer` | `Drop` / RAII |
| `recover()` | `catch_unwind` |
| `context.Context` | `tokio_util::sync::CancellationToken` |
| functional options | builder methods |
| `time.Ticker` | `tokio::time::interval` |
| `os/signal` | `tokio::signal` |

## Not yet ported (tracked by phase)

- Input: keys/mouse/focus/paste — Phase 2
- Diff renderer (`cursed_renderer.go`) — Phase 3
- Extended commands, clipboard, screen — Phase 4
- Color detection/degrade, full `View` — Phase 5
- exec, signals, suspend/resume, logging — Phase 6
