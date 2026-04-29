# AGENT.md

This file is a session handoff for future agent runs.
Use it to quickly understand the repo, prior decisions, and safe workflows.

## Project Identity

- Project: `rustyquarium`
- Language: Rust
- Rendering library: `crossterm` (v0.28)
- Crate name: `rustyquarium`
- Entry point: `src/main.rs`

## Current Repo Layout

```
src/
  main.rs        — process entry point
  cli.rs         — flag parsing, app dispatch, and startup scene composition
  animation.rs   — runtime loop, event handling, draw pipeline
  entity.rs      — entity model, movement, lifecycle, collisions
  environment.rs — water, castle, seaweed
  fish.rs        — fish / bubbles / splat logic
  special.rs     — shark / ship / whale / monster / hook / ducks / dolphins / swan
  depth.rs       — z-layer constants
  info.rs        — user-facing text, version string, and VERSION constant
```

Non-code:

- `Cargo.toml`, `Cargo.lock`
- `Makefile`
- `README.md`
- `Entities.md` — visual entity catalog grouped by class
- `AGENT.md` — this file
- `LICENSE`

## Key Design Decisions

### Entity references use `Rc<RefCell<Entity>>`

Go's pointer semantics are replicated with `type EntityRef = Rc<RefCell<Entity>>`.
This lets collision lists hold real entity references, callbacks mutate any entity,
and `del_entity` uses `Rc::ptr_eq` for identity — all without lifetime fights.

### Update loop uses `std::mem::take`

During the movement pass, `self.entities` is temporarily replaced with an empty Vec:

```rust
let mut entities = std::mem::take(&mut self.entities);
for eref in entities.clone() {
    // callbacks can call add_entity — writes into the now-empty self.entities
}
let spawned = std::mem::take(&mut self.entities);
entities.extend(spawned);
self.entities = entities;
```

New entities spawned by callbacks accumulate in `self.entities` and are merged back
after the loop. This mirrors Go's `append([]*Entity{}, a.entities...)` copy trick.

### Callback types

| Purpose          | Type                                          |
|------------------|-----------------------------------------------|
| Movement/update  | `fn(EntityRef, &mut Animation)` (fn pointer)  |
| Collision        | `fn(EntityRef, &mut Animation)` (fn pointer)  |
| Death            | `Box<dyn Fn(EntityRef, &mut Animation)>`      |

Death callbacks are closures because some capture values (e.g. `classic` flag in
`add_fish`'s respawn closure). Movement/collision callbacks are plain fn pointers
because they capture nothing.

### CallbackArgs enum

Go's `any` polymorphism for callback arguments is replaced by:

```rust
pub enum CallbackArgs {
    Move(Vec<f64>),                    // [dx, dy, dz, frame_step]
    State(HashMap<String, String>),    // fishhook state machine
}
```

### Terminal I/O

`crossterm` replaces `tcell`. Key equivalences:

| Go / tcell                       | Rust / crossterm                        |
|----------------------------------|-----------------------------------------|
| `tcell.NewScreen()` + `s.Init()` | `enable_raw_mode()` + `EnterAlternateScreen` |
| `s.Fini()`                       | `disable_raw_mode()` + `LeaveAlternateScreen` |
| `s.Size()`                       | `terminal::size()`                      |
| `s.SetContent(x, y, ch, style)`  | `queue!(MoveTo, SetForegroundColor, Print)` |
| `s.Show()`                       | `stdout.flush()`                        |
| `s.HasPendingEvent()`            | `event::poll(Duration::ZERO)`           |
| `s.PollEvent()`                  | `event::read()`                         |

### No goroutines

The Go version used a goroutine to work around `PollEvent` blocking.
In Rust, `event::poll(Duration::ZERO)` is non-blocking, so no threads are needed.
The main loop is fully sequential.

## Build / Test Workflow

```bash
cargo build          # debug build
cargo build --release
cargo test           # run tests (currently none; see below)
make                 # release build via Makefile
make install         # install to $HOME/.local/usr/bin/rustyquarium
make uninstall
```

Smoke tests:

```bash
./target/debug/rustyquarium --version
./target/debug/rustyquarium --info
```

## Known Warnings

Three unused constants in `depth.rs` (`DEPTH_GUI_TEXT`, `DEPTH_GUI`, `DEPTH_WATER_GAP0`)
and two in `version.rs` (`ORIGINAL_AUTHOR`, `ORIGINAL_PROJECT`) are intentionally kept
for completeness / parity with the Go source. They may be suppressed with `#[allow(dead_code)]`
if they become noise.

## Behavioral Notes for Future Changes

- `Entity.callback_args` is a `CallbackArgs` enum:
  - `Move(vec![dx, dy, dz, frame_step])` — standard movement
  - `State(map)` — fishhook state machine (`"mode" => "lowering"` / `"hooked"`)
- Collision pass is O(n²) AABB, intentionally simple.
- Render order is by `z` descending; collision does not use depth.
- Many special entities chain via death callbacks into `random_object`.
- Fish respawn closure captures `classic: bool` — this is why death callbacks are
  `Box<dyn Fn>` rather than plain fn pointers.

## If Starting a New Session

1. Run `cargo build` to verify baseline.
2. Run `./target/debug/rustyquarium --version` and `--info` as smoke tests.
3. For visual issues, inspect sprite strings in `fish.rs` / `special.rs` and
   the draw path in `animation.rs` (`draw_entity`).
4. Use `Entities.md` to locate any entity's spawn path and behavior.
5. For parity questions, compare against the Go source in `../goquarium/`.
