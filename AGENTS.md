# AGENTS.md - Coding Agent Guidelines for breakout-rust

## Project Overview

Breakout game built with Rust and Bevy 0.19. Single crate with a library and a
binary target, no workspace.
Rust edition 2024. Single dependency: `bevy`. No feature flags, no build scripts.

## Build / Run / Test Commands

```sh
cargo build                  # Dev build
cargo build --release        # Release build
cargo run                    # Run the game
cargo run -- --headless      # Run the simulation headless (training mode)
cargo check                  # Type-check only (fastest feedback)
cargo clippy -- -D warnings  # Lint (treat warnings as errors)
cargo fmt                    # Format code
cargo fmt -- --check         # Check formatting without modifying
cargo test                   # Run all tests (94 tests across 7 modules)
cargo test <test_name>       # Run a single test by name
cargo test --lib <module>::tests::<test_name>  # Single test in module
cargo add <crate_name>       # Add dependency (never edit Cargo.toml manually)
```

## Project Structure

```
src/
  main.rs           # Thin entry point: windowed game or headless training mode
  lib.rs            # Module declarations (alphabetical), PresentationPlugin, game_app()
  background.rs     # Self-contained BackgroundPlugin (shader material + systems)
  collision.rs      # Collision detection systems (AABB-based)
  components.rs     # All shared types, resources, constants, collision helper
  environment.rs    # Headless app, BreakoutEnv (reset/step/observation), Observation
  game.rs           # UI updates, state transitions, restart, pause menu logic
  gameplay.rs       # GameplayPlugin + `Simulation` schedule (fixed-timestep physics)
  movement.rs       # PaddleAction input mapping, paddle and ball movement
  setup.rs          # Spawn/despawn systems: camera, entities, UI, overlays
assets/
  shaders/
    background.wgsl # WGSL fragment shader for animated background
```

Flat module structure - one file per module, no nested `mod.rs` directories.

## Code Style

### Formatting & Linting

Default `rustfmt` (no `rustfmt.toml`) and default `clippy` (no `clippy.toml`).
Use `#[allow(clippy::type_complexity)]` and `#[allow(clippy::too_many_arguments)]`
on Bevy systems with complex query types - never globally.

### Naming Conventions

| Item              | Convention            | Example                           |
|-------------------|-----------------------|-----------------------------------|
| Files             | `snake_case.rs`       | `collision.rs`                    |
| Structs / Enums   | `CamelCase`           | `GameState`, `BackgroundMaterial` |
| Functions         | `snake_case`          | `spawn_game`, `move_paddle`       |
| Constants         | `SCREAMING_SNAKE_CASE`| `WINDOW_WIDTH`, `BALL_SPEED`      |
| Variables         | `snake_case`          | `ball_pos`, `grid_width`          |
| UI markers        | Suffix `Ui` (not UI)  | `ScoreboardUi`, `LivesUi`        |

### Imports

Two groups separated by a blank line: external first, then crate-internal.

```rust
use bevy::prelude::*;

use crate::components::*;
```

- **Glob import** `bevy::prelude::*` and `crate::components::*` when many items needed.
- **Selective imports** when only a few items: `use crate::components::{WINDOW_HEIGHT, WINDOW_WIDTH};`
- Non-prelude bevy items get their own `use` lines: `use bevy::app::AppExit;`
- No `use std::*` - access std items by full path (e.g., `std::f32::consts::FRAC_PI_4`).
- In `main.rs` (crate root), use `use components::*` without `crate::` prefix.

### Constants

All shared constants live in `components.rs` under `// --- Shared Constants ---`.
Always `pub const` with explicit type annotations. Grouped by domain with
line-comment headers (`// Window`, `// Paddle`, `// Ball`, `// Bricks`, `// Walls`).
Colors use `Color::srgb(r, g, b)`. Never use `static`.

### Types and Derives

- **Marker components**: `#[derive(Component)]` only (e.g., `Paddle`, `Brick`, `Wall`).
- **Data components**: named `pub` fields with `#[derive(Component)]` (e.g., `Ball`).
- **Resources**: `#[derive(Resource, Default)]` when zero-default is fine;
  manual `impl Default` only when custom defaults are needed (e.g., `Lives { count: 3 }`).
- **Game state**: `#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]`.
- **Custom materials**: `#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]`.
- Only derive what is actually used - minimal derive sets.

### Error Handling

- **No `unwrap()`, `expect()`, or `panic!()`** in production code (allowed in tests).
- **`let Ok(...) = query.single_mut() else { return; }`** for fallible singleton queries.
- **`if let Ok(...)`** for less critical query results.
- **`if let Some(...)`** for Option values (collision results).
- **`saturating_sub`** for safe decrement without underflow.
- **Early return** with guard clauses: `if !resource.is_changed() { return; }`.
- Systems return `()` - no `Result` return types.

### Comments

- **Doc comments (`///`)** on every `pub fn`: single-line, starts with verb.
- **Section headers** in `components.rs`: `// --- Section Name ---` with triple dashes.
- **Inline comments** above code blocks for context: `// Paddle`, `// Ball`.
- **No module-level `//!` doc comments**.

## Bevy-Specific Patterns

**System parameter order**: Input resources (`Res<ButtonInput<KeyCode>>`, `Res<Time>`)
-> Mutable state (`ResMut<NextState<...>>`, `Commands`) -> Queries (`Query<...>`).
Exception: `Commands` comes first in spawn-focused systems.

**App builder order in `main.rs`**:
1. `add_plugins(DefaultPlugins.set(...))` then custom plugins
2. `.init_state::<T>()` / `.init_resource::<T>()`
3. `.add_systems(Startup, ...)` then state-grouped systems
4. Each state group labeled with comments: `// Menu state`, `// Playing state`, etc.

**System ordering**: Use `.chain()` for sequential execution within a state group.
**Multi-state runs**: Use `.or()` - e.g., `in_state(Playing).or(in_state(Paused))`.
**State hooks**: `OnEnter(GameState::X)` / `OnExit(GameState::X)` for spawn/despawn.
**Entity spawning**: Tuple bundles `commands.spawn((Component, Component, ...))`.
**Hierarchical UI**: `with_children` / `with_child` for nested UI elements.
**Sprite sizing**: `Sprite { custom_size: Some(Vec2::new(...)), ..default() }`.
**z-ordering**: background `-100.0`, default entities `0.0`, ball `1.0`.
**First-run guard**: `Local<bool>` to skip logic on first state enter.
**App exit**: `MessageWriter<AppExit>` (Bevy 0.18 API, not `EventWriter`).
**Let-chains**: Rust 2024 edition enables `if let ... && condition { }`.

**Simulation vs presentation**: `GameplayPlugin` owns the headless-safe simulation
in the custom `Simulation` schedule; `PresentationPlugin` owns window, rendering,
UI and player input. The windowed app drives `Simulation` from `FixedUpdate`,
while `BreakoutEnv` steps it manually. Gameplay systems use the `SIM_DT`
constant, never `Time::delta_secs()`.

**Actions**: keyboard input writes the `PaddleAction` resource; only
`movement::move_paddle` reads it, so agents can drive the paddle the same way.

**Plugins** are self-contained: own their types, startup systems, and update systems.
Plugin-internal functions are private; cross-module functions are `pub`.

## Testing Patterns

Tests live in `#[cfg(test)] mod tests` at the bottom of each file. Key conventions:

- `test_app()` helper creates `App::new()` with `MinimalPlugins` for minimal Bevy setup.
- `game.rs` has a separate `pause_menu_test_app()` for pause-specific tests.
- Setup: `app.world_mut().spawn(...)` -> Run: `app.update()` -> Assert: query results.
- `unwrap()` is allowed in test code (never in production).
- Section comments within test modules: `// --- function_name ---`.

## Git Conventions

- **Default branch**: `main`
- **Feature branches**: `feat/<feature-name>` (kebab-case)
- **Commit format**: `type: short description` (lowercase, imperative, no period)
  - Types: `feat:`, `fix:`, `refactor:`, `docs:`, `chore:`
  - Body separated by blank line, uses `-` bullet lists

## Architecture

- `components.rs` is the shared "prelude" - holds all types other modules need.
- AABB collision via `check_aabb_collision()` - reused by wall, paddle, and brick systems.
- Game states: `Menu -> Playing <-> Paused`, `Playing -> GameOver | Victory -> Menu`.
- No `unsafe`, no `async`, no logging/tracing instrumentation.
