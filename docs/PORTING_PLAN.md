# Bubble Tea → Rust Port Plan

A full port of [Bubble Tea v2](https://charm.land/bubbletea) (`charm.land/bubbletea/v2`)
to Rust, preserving The Elm Architecture (Model / Update / View) and the public
API shape, while replacing Go-specific machinery with idiomatic Rust equivalents.

Source of truth: `../bubbletea` (Go v2, ~5.7k LOC across 40 files).

## Locked Design Decisions

| Concern | Go | Rust port | Rationale |
|---|---|---|---|
| Message type | `Msg = any` (`uv.Event`) | `type Msg = Box<dyn Any + Send>` | Closest port; preserves component composition (each component emits its own message types, matched via downcast). |
| Concurrency | goroutines + channels | **tokio** tasks + `mpsc` channels | Ergonomic async I/O; `Cmd` becomes an async future. |
| Command | `Cmd func() Msg` | `type Cmd = Pin<Box<dyn Future<Output = Option<Msg>> + Send>>` | Async-returning-message; `None` ≈ Go `nil`. |
| Terminal I/O | `ultraviolet` + `x/ansi` + `x/term` | **crossterm** crate | Raw mode, key/mouse parsing, cross-platform, async event stream. |
| Diff renderer | `cursed_renderer.go` | Ported by hand → `renderer/standard.rs` | No direct crate equivalent; port the cell-buffer diff logic. |
| Cell width | `displaywidth`/`uniseg` | `unicode-width` + `unicode-segmentation` | Grapheme + east-asian width. |
| Color profile | `colorprofile` | `anstyle` + small ported degrade logic | Truecolor → ANSI256 → ANSI16 fallback. |
| Builder options | functional opts `WithX(p)` | builder pattern on `Program` | Idiomatic Rust. |
| Error handling | `error` returns | `thiserror` enums + `Result` | Idiomatic. |

## Crate Layout

```
bubbletea-rs/
  Cargo.toml
  src/
    lib.rs            # public exports: Model, Msg, Cmd, View, Program
    model.rs          # Model trait
    message.rs        # Msg alias, downcast helpers, internal msgs
    command.rs        # Cmd, batch, sequence, tick, every, quit, ...
    program/
      mod.rs          # Program struct + builder
      run.rs          # async run loop (event_loop equivalent)
      signals.rs      # SIGINT/SIGTERM/SIGWINCH, suspend (ctrl+z)
    key.rs            # KeyPressMsg/KeyReleaseMsg, key parsing via crossterm
    mouse.rs          # MouseMsg variants, MouseMode
    screen.rs         # altscreen, clear, scroll region commands/msgs
    cursor.rs         # Cursor, CursorShape
    focus.rs          # FocusMsg/BlurMsg
    clipboard.rs      # OSC52 read/set clipboard
    exec.rs           # ExecProcess (suspend TUI, run child, resume)
    color.rs          # color profile + degradation
    options.rs        # ProgramOption equivalents (builder methods)
    logging.rs        # log-to-file helper
    view.rs           # View struct (Content, Cursor, AltScreen, MouseMode, ...)
    terminal.rs       # raw mode enter/leave, tty detection (thin over crossterm)
    renderer/
      mod.rs          # Renderer trait
      standard.rs     # diff/cell-buffer renderer (port of cursed_renderer)
      nil.rs          # no-op renderer
      cellbuf.rs      # Cell, CellBuffer, width-aware diff
  examples/
    simple.rs, altscreen_toggle.rs, spinner.rs, ...
  docs/
    PORTING_PLAN.md   # this file
    API_MAPPING.md    # Go symbol → Rust symbol table (living doc)
```

## Public API Target

```rust
use bubbletea_rs as tea;

#[derive(Default)]
struct Model { count: i32 }

impl tea::Model for Model {
    fn init(&mut self) -> Option<tea::Cmd> { Some(tick()) }

    fn update(&mut self, msg: tea::Msg) -> Option<tea::Cmd> {
        if let Some(k) = msg.downcast_ref::<tea::KeyPressMsg>() {
            match k.to_string().as_str() {
                "ctrl+c" | "q" => return Some(tea::quit()),
                _ => {}
            }
        }
        if msg.is::<TickMsg>() {
            self.count -= 1;
            if self.count <= 0 { return Some(tea::quit()); }
            return Some(tick());
        }
        None
    }

    fn view(&self) -> tea::View {
        tea::View::new(format!("Exiting in {} seconds...", self.count))
    }
}

#[tokio::main]
async fn main() -> tea::Result<()> {
    tea::Program::new(Model { count: 5 }).run().await?;
    Ok(())
}
```

Note: Go returns `(Model, Cmd)` from `Update`; the Rust port takes `&mut self`
and returns `Option<Cmd>` (mutation in place — idiomatic and avoids cloning the
whole model each tick).

## Phases

Each phase is independently compilable and testable. Phases marked **[sonnet]**
are mechanical ports suitable for the cheaper model; **[opus]** require design
judgment.

### Phase 0 — Scaffolding **[opus]**
- Cargo.toml with deps: `tokio` (rt-multi-thread, macros, sync, time),
  `crossterm` (event-stream), `unicode-width`, `unicode-segmentation`,
  `anstyle`, `thiserror`, `futures`.
- `lib.rs` skeleton, `model.rs` trait, `message.rs`, `command.rs` minimal
  (`quit`, `batch`, `sequence`).
- Goal: `cargo build` green with a do-nothing Program.

### Phase 1 — Core runtime **[opus]**
- `Program` struct + builder (`options.rs`).
- `run.rs`: async event loop — receive msgs, run `update`, dispatch cmds as
  tokio tasks, render. Port of `eventLoop`, `handleCommands`, `Send`, `Quit`,
  `Kill`, `Wait`, batch/sequence execution, panic recovery (catch_unwind).
- `terminal.rs`: enter/leave raw mode, alt screen, restore on drop (RAII guard
  replaces Go's deferred `shutdown`).
- Goal: countdown example runs and quits cleanly.

### Phase 2 — Input **[opus for parsing design, sonnet for tables]**
- `key.rs`: map crossterm `KeyEvent` → `KeyPressMsg`/`KeyReleaseMsg`, including
  `to_string()` formatting ("ctrl+c", "shift+enter", "alt+a", function keys).
  The key-name string table is mechanical **[sonnet]**.
- `mouse.rs`: crossterm `MouseEvent` → `MouseClickMsg`/`Release`/`Wheel`/`Motion`,
  `MouseMode`.
- `focus.rs`, paste (bracketed paste) → `PasteMsg`.
- Goal: `print-key` style example echoes keys/mouse correctly.

### Phase 3 — Renderer **[opus]**
- `cellbuf.rs`: width-aware `Cell`/`CellBuffer`, line diffing.
- `standard.rs`: port `cursed_renderer.go` — frame diffing, cursor movement
  optimization (hard tabs/backspace), alt-screen vs inline, `insert_above`
  (Println), synchronized output (mode 2026), scroll handling.
- `nil.rs`: no-op renderer.
- Wire the 60fps ticker (tokio interval) → flush.
- Goal: pixel-identical frames vs Go golden tests where feasible.

### Phase 4 — Commands & messages **[sonnet]**
- `command.rs`: `tick`, `every`, `batch`, `sequence`, `tea::printf/println`,
  window-title, set background/foreground/cursor color, request capability,
  enable/disable mouse, report focus, keyboard enhancements.
- `screen.rs`: clear screen, enter/exit alt screen, scroll.
- `clipboard.rs`: OSC52 read/set system + primary clipboard.
- Mostly thin wrappers emitting internal msgs — mechanical.

### Phase 5 — Color & styled content **[opus]**
- `color.rs`: color profile detection (env-based) + truecolor→256→16 degrade.
- `view.rs`: full `View` (BackgroundColor, ForegroundColor, WindowTitle,
  ProgressBar, KeyboardEnhancements, OnMouse hook).
- Decide lipgloss interop: the sibling `../lipgloss` styled strings are ANSI —
  renderer consumes ANSI-encoded content, so no hard dependency. (Port lipgloss
  separately later if desired.)

### Phase 6 — Process control & platform **[opus]**
- `exec.rs`: release terminal, run child process, restore (port `exec.go`).
- `signals.rs`: SIGINT/SIGTERM via tokio signal, SIGWINCH resize, ctrl+z suspend
  (`SIGTSTP`) + resume. Windows equivalents behind `cfg`.
- `logging.rs`: `log_to_file`.

### Phase 7 — Examples & tests **[sonnet]**
- Port representative examples: simple, altscreen-toggle, spinner, mouse,
  progress, list-simple, exec, send-msg, fullscreen, print-key.
- Port unit tests (`tea_test.go`, `screen_test.go`, `commands_test.go`,
  `cursed_renderer_test.go`) to Rust `#[test]` / `#[tokio::test]`.

## Go → Rust Feature Substitutions

| Go feature | Rust replacement |
|---|---|
| `interface{}` / type switch | `Box<dyn Any>` + `downcast_ref` / `is::<T>()` |
| goroutine | `tokio::spawn` |
| channel `chan T` | `tokio::sync::mpsc` / `oneshot` |
| `defer` cleanup | `Drop` impl / RAII guards |
| `recover()` panic catch | `std::panic::catch_unwind` + `AssertUnwindSafe` |
| `context.Context` cancel | `tokio_util::sync::CancellationToken` |
| `sync.Once` | `std::sync::Once` / `OnceLock` |
| `sync.WaitGroup` | `tokio::task::JoinSet` |
| functional options | builder methods |
| `time.Ticker` | `tokio::time::interval` |
| `os/signal` | `tokio::signal` |

## Open Questions / Risks

- **Model ownership**: `&mut self` update vs Go's value-returning `Update`.
  Recommended `&mut self` (less cloning); flagged for confirmation.
- **`bubbles` components** (separate Go repo) are out of scope for this port but
  the `Box<dyn Any>` decision keeps the door open.
- **Windows fidelity**: crossterm covers it; lower test priority than Unix.
- **Golden-test parity**: exact byte-for-byte ANSI output may differ from Go due
  to crossterm's sequence choices; we assert visual/cell equivalence instead.

## Status Tracking

Progress is tracked in the task list (this session) and reflected by which
phases compile + pass tests. `API_MAPPING.md` is updated as symbols land.
