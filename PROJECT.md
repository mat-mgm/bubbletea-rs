# Bubble Tea → Rust Port

## Overview
A full port of [Bubble Tea v2](https://charm.land/bubbletea) (`charm.land/bubbletea/v2`,
the Go TUI framework based on The Elm Architecture) to Rust.

* **Purpose**: Provide a native Rust TUI framework with the same Model/Update/View
  programming model as Bubble Tea, so the architecture, ergonomics, and example
  programs carry over. Where a Go feature has no direct Rust analogue, it is
  replaced with an idiomatic equivalent rather than dropped.
* **Context**: Go source of truth lives at `../bubbletea` (v2, ~5.7k LOC, 40 `.go`
  files). Go-specific machinery (goroutines, `interface{}`, `defer`, functional
  options) maps onto Rust equivalents (tokio, `Box<dyn Any>`, `Drop`, builders).
  Terminal I/O that Go gets from `ultraviolet`/`x-ansi`/`x-term` is provided by
  `crossterm`.
* **Scope**:
  * **In**: the `tea` package itself — runtime/event loop, Model trait, commands,
    input (key/mouse/focus/paste), renderer, screen/cursor control, color profile,
    exec/process control, signals, logging, representative examples + tests.
  * **Out**: the separate `bubbles` component library and `lipgloss` styling
    library (the `Box<dyn Any>` message decision keeps the door open to port
    `bubbles` later; the renderer consumes ANSI-encoded content so `lipgloss`
    is not a hard dependency).
* **References**: `docs/PORTING_PLAN.md` (detailed module mapping & phase notes),
  `docs/API_MAPPING.md` (living Go→Rust symbol table, created in Phase 0).

## Status
Current status: in-progress
Start date: 2026-05-29
Last updated: 2026-05-29
Priority: normal

## Goals
* **Functional parity** with Bubble Tea v2 for the core `tea` package: any program
  expressible against the Go API has a faithful Rust equivalent.
* **Idiomatic Rust**: async via tokio, RAII terminal cleanup, builder configuration,
  `Result`-based errors — not a line-by-line transliteration.
* **Verifiable**: ported unit tests pass; representative examples run correctly on
  a real terminal (Unix priority, Windows best-effort via crossterm).

Success criteria:
* The countdown/`simple` example and a set of representative examples
  (altscreen-toggle, spinner, mouse, progress, exec, send-msg, print-key) run
  correctly.
* Ported tests (`tea_test`, `screen_test`, `commands_test`, `cursed_renderer_test`)
  pass as Rust `#[test]`/`#[tokio::test]`.
* `cargo build` and `cargo clippy` are warning-free.

Constraints / priorities:
* Prefer simple, composable, minimal-dependency design (suckless-leaning).
* Preserve the public API shape where Rust allows; document every deliberate
  divergence in `docs/API_MAPPING.md`.

## Development Guidelines
* **Environment**: Provide a Nix flake (`nix develop`) pinning the Rust toolchain
  (rustc 1.91, cargo) and any native deps. Enter the shell once and develop inside
  it. *(Flake not yet created — Phase 0 task.)*
* **Version Control**: Use `git`. This directory is **not yet a git repo** —
  initialize it in Phase 0. Commit after each phase is implemented and verified.
* **Workflow**: After each phase, wait for explicit user confirmation before
  ticking verification checks and committing.
* **Style**: suckless coding style; robust, debuggable code over cleverness.
* **Warnings**: fix all `rustc`/`clippy` warnings.
* **Roadmap Expansion**: once all defined phases are complete, expand with
  follow-on phases (e.g. `lipgloss`/`bubbles` ports, Windows hardening).

## Architecture
* **Structure**: single crate `bubbletea-rs`; modules mirror the Go file layout.
  Public surface re-exported from `lib.rs`: `Model`, `Msg`, `Cmd`, `View`,
  `Program`, message/command types.
* **Key components**:
  * `program/` — `Program` struct + builder, async run loop (`eventLoop` port),
    signal handling, command dispatch.
  * `model.rs` — `Model` trait (`init`/`update`/`view`).
  * `message.rs` / `command.rs` — `Msg = Box<dyn Any + Send>`, `Cmd` futures,
    `batch`/`sequence`/`tick`/`every`/`quit`.
  * `renderer/` — `Renderer` trait, hand-ported diff/cell-buffer renderer
    (`standard.rs` from `cursed_renderer.go`), `nil.rs`.
  * `key.rs` / `mouse.rs` / `focus.rs` — crossterm event → `tea` message mapping.
  * `view.rs` / `screen.rs` / `cursor.rs` / `color.rs` — view model + screen,
    cursor, color-profile control.
  * `exec.rs` / `terminal.rs` / `logging.rs` — process control, raw-mode RAII,
    log-to-file.
* **Resources (external deps)**: `tokio`, `crossterm`, `unicode-width`,
  `unicode-segmentation`, `anstyle`, `thiserror`, `futures`.

## Roadmap

### General conditions
Develop inside `nix develop`. Rust 1.91 / cargo. Each phase must compile
warning-free and be committed once verified. Go source of truth: `../bubbletea`.
Status legend: `[ ]` todo · `[~]` in-progress · `[✓]` done · `[x]` blocked ·
`[?]` optional · `[!]` critical.

**Design decisions (project-wide, locked 2026-05-29)**
- Decision: `Msg` = `Box<dyn Any + Send>` (dynamic), matched via `downcast_ref`.
  Rationale: closest port of Go's `interface{}`; preserves component composition
  where each component emits its own message types.
  Alternatives: generic associated `Msg` enum (type-safe but diverges from
  Bubble Tea semantics and complicates reusable components).
- Decision: tokio async runtime; `Cmd = Pin<Box<dyn Future<Output=Option<Msg>> + Send>>`.
  Rationale: ergonomic async I/O; `None` ≈ Go `nil`.
  Trade-off: colors command code async; adds a large dependency.
- Decision: `crossterm` for terminal I/O (raw mode, key/mouse parsing, async event
  stream, cross-platform). Renderer's diff logic is hand-ported.
  Alternative: from-scratch port of `ultraviolet` (max fidelity, far more effort).
- Decision: `Model::update(&mut self) -> Option<Cmd>` (mutate in place) instead of
  Go's `Update(Msg) (Model, Cmd)` value return. **Needs user confirmation.**
  Rationale: avoids cloning the whole model per message.

### Phase 0: Scaffolding [✓]
**Description**: Make `bubbletea-rs` a real, buildable crate with the public
skeleton in place; set up git + Nix.

**Tasks**
- [✓] `Cargo.toml` with locked dependencies
- [✓] `lib.rs` public exports skeleton
- [✓] `model.rs` `Model` trait
- [✓] `message.rs` (`Msg` alias + downcast helpers + internal msgs)
- [✓] `command.rs` minimal (`quit`, `batch`, `sequence`, `tick`)
- [✓] `flake.nix` dev shell (rust toolchain)
- [✓] `git init` + initial commit
- [✓] `docs/API_MAPPING.md` started

**Checks**
- [✓] `cargo build` green with a working `Program` (countdown example runs, exit 0)
- [~] `cargo clippy` warning-free — clippy unavailable (no rustup in env); `cargo build` is warning-free. Run `cargo clippy` inside `nix develop`.
- [?] `nix develop` enters a working shell — not verified in this env (nix not invoked); flake provided.

### Phase 1: Core runtime [✓]
**Description**: Async event loop, `Program` + builder, terminal RAII, command
dispatch, panic recovery. Port of `tea.go` (`eventLoop`, `handleCommands`, `Send`,
`Quit`/`Kill`/`Wait`, batch/sequence) and `options.go`.

**Tasks**
- [✓] `Program` struct + builder (`options.rs`)
- [✓] `program/run.rs` async event loop + command dispatch as tokio tasks
- [✓] `terminal.rs` raw-mode RAII guard + size detection (replaces deferred `shutdown`)
- [✓] panic recovery via `catch_unwind`; cancellation via `CancellationToken`
      (`cancellation_token()` / `Sender::quit` replace `WithContext`/`p.Quit`)
- [✓] renderer flush ticker (`tokio::time::interval`, fps-paced)

**Checks**
- [✓] countdown example runs and quits cleanly (timer path; ctrl+c/q need Phase 2 input)
- [✓] `tests/runtime.rs`: batch, sequence-in-order, external-sender-quit all pass

**Dependencies**: Phase 0.

**Notes**: alt-screen-from-`View` is wired via `TerminalGuard::set_alt_screen` but
exercised properly once the diff renderer (Phase 3) lands. `WithFilter` deferred
(needs model-aware filter design); tracked for a later phase.

### Phase 2: Input [✓]
**Description**: Map crossterm events to `tea` messages. Port `key.go`,
`keyboard.go`, `mouse.go`, `focus.go`, bracketed paste.

**Tasks**
- [✓] `key.rs` `KeyPressMsg`/`KeyReleaseMsg` + `Display` keystroke formatting
- [✓] key-name string table + 6 formatting unit tests
- [✓] `mouse.rs` mouse messages + `MouseMode`
- [✓] `focus.rs` focus/blur; paste → `PasteMsg`
- [✓] `input.rs` async crossterm `EventStream` reader task (cancellable)
- [✓] view-driven terminal modes (mouse/focus/bracketed-paste) in `TerminalGuard`

**Checks**
- [✓] key-formatting unit tests pass (ctrl/alt/shift order, special keys, space)
- [✓] `print_key` + updated `simple` examples build
- [MANUAL] interactive echo of real keys/mouse/focus — needs a TTY; batched into
  the post-Phase-3 manual check.

**Dependencies**: Phase 1.

### Phase 3: Renderer [✓] [!]
**Description**: Hand-port the diff renderer. Port `cursed_renderer.go`,
`renderer.go`, `nil_renderer.go`.

**Tasks**
- [✓] `renderer/cellbuf.rs` ANSI-aware width measurement + line splitter (unit tests)
- [✓] `renderer/standard.rs` line-diffing: inline (cursor-up+rewrite) and alt-screen (home+fill), force-repaint on resize, insert_above, cursor hide/show
- [✓] `renderer/nil.rs` no-op
- [✓] 60fps flush ticker (`tokio::time::interval`)

**Checks**
- [✓] cellbuf unit tests pass; countdown renders and updates in-place correctly
- [MANUAL] alt-screen + full session — batched into post-Phase-3 manual check

**Notes**: Line-level diff (simpler than Go's cell-level ultraviolet diff);
adequate for correctness, can be upgraded later.

**Dependencies**: Phase 1.

### Phase 4: Commands & messages [ ] [sonnet]
**Description**: Thin wrappers emitting internal messages. Port `commands.go`,
`screen.go`, `clipboard.go`.

**Tasks**
- [ ] `command.rs` `tick`, `every`, `println`/`printf`, window title, set/request
      colors, mouse enable/disable, report focus, keyboard enhancements
- [ ] `screen.rs` clear screen, enter/exit alt screen, scroll
- [ ] `clipboard.rs` OSC52 system + primary clipboard

**Checks**
- [ ] ported `commands_test` + `screen_test` pass

**Dependencies**: Phase 1, 3.

### Phase 5: Color & styled content [ ]
**Description**: Color profile + degradation, full `View`. Port `color.go`,
`profile.go`, remaining `View` fields from `tea.go`.

**Tasks**
- [ ] `color.rs` profile detection + truecolor→256→16 degrade
- [ ] `view.rs` full `View` (bg/fg color, window title, progress bar, keyboard
      enhancements, `OnMouse` hook)
- [ ] confirm lipgloss interop strategy (ANSI content; no hard dep)

**Checks**
- [ ] colors degrade correctly under forced `NO_COLOR`/256/16 profiles

**Dependencies**: Phase 3.

### Phase 6: Process control & platform [ ]
**Description**: Port `exec.go`, signals (`signals_unix.go`/`signals_windows.go`),
suspend/resume, `logging.go`.

**Tasks**
- [ ] `exec.rs` release terminal → run child → restore
- [ ] `program/signals.rs` SIGINT/SIGTERM (tokio signal), SIGWINCH resize,
      ctrl+z suspend (SIGTSTP) + resume; Windows behind `cfg`
- [ ] `logging.rs` `log_to_file`

**Checks**
- [ ] `exec` example launches `$EDITOR` and restores cleanly
- [ ] resize + ctrl+z/fg round-trip works

**Dependencies**: Phase 1, 3.

### Phase 7: Examples & tests [ ] [sonnet]
**Description**: Port representative examples and unit tests.

**Tasks**
- [ ] examples: simple, altscreen-toggle, spinner, mouse, progress, list-simple,
      exec, send-msg, fullscreen, print-key
- [ ] port `tea_test`, `screen_test`, `commands_test`, `cursed_renderer_test`

**Checks**
- [ ] examples run; ported test suite green

**Dependencies**: Phases 1–6.

**Notes / Risks**
- Byte-for-byte ANSI parity with Go is unlikely (crossterm picks its own
  sequences); assert visual/cell equivalence instead.
- `Model` ownership (`&mut self` vs value-return) is pending user confirmation —
  resolve before Phase 1 lands.
- Windows is best-effort; Unix is the verification priority.
