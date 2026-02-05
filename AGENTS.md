# AGENTS.md — Coding Agent Guidelines for breakout-rust

## Project Overview

Breakout game built with Rust and Bevy 0.18. Single-crate binary, no workspace.
Rust edition 2024. Single dependency: `bevy`.

## Build / Run / Test Commands

```sh
# Build (dev)
cargo build

# Build (release, optimized)
cargo build --release

# Run the game
cargo run

# Check without building artifacts (faster feedback)
cargo check

# Lint with clippy
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run all tests
cargo test

# Run a single test by name
cargo test <test_name>

# Run tests in a specific module
cargo test --bin breakout-rust <module>::tests::<test_name>

# Add a dependency
cargo add <crate_name>
```

No custom profiles, no feature flags, no build scripts.
`Cargo.lock` is committed (binary crate).

## Project Structure

```
src/
  main.rs           # App entry, module declarations, Bevy App builder
  background.rs     # Self-contained BackgroundPlugin (shader material + systems)
  collision.rs      # Collision detection systems
  components.rs     # Components, resources, GameState, constants, shared helpers
  game.rs           # Game logic: UI updates, state transitions, restart
  movement.rs       # Movement systems: paddle input, ball physics
  setup.rs          # Spawn/despawn systems: camera, entities, UI, overlays
assets/
  shaders/
    background.wgsl # WGSL fragment shader for animated background
docs/
  doc/
    breakout_rust/   # Generated rustdoc API documentation
```

Flat module structure — one file per module, no nested `mod.rs` directories.
Modules declared in `main.rs` in **alphabetical order**.

## Documentation

Generated API documentation (`cargo doc` output) is available in `docs/`.
Browse project-specific docs at `docs/doc/breakout_rust/`. To regenerate:

```sh
CARGO_TARGET_DIR=docs cargo doc --no-deps
```

## Code Style

### Formatting

Default `rustfmt` — no `rustfmt.toml`. 4-space indent, trailing commas in
multi-line constructs. No `clippy.toml` either — default clippy rules apply.

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

Two groups, separated by a blank line:

1. External crate imports (`bevy::*`)
2. Crate-internal imports (`use crate::components::*`)

```rust
use bevy::prelude::*;

use crate::components::*;
```

- **Glob import** `bevy::prelude::*` and `crate::components::*` when many items needed.
- **Selective imports** when only a few items needed: `use crate::components::{WINDOW_HEIGHT, WINDOW_WIDTH};`
- Non-prelude bevy items get their own `use` lines.
- No `use std::*` — rely on Bevy re-exports; access std items by path if needed.

### Constants

- Always `pub const`, never `static`.
- All shared constants live in `components.rs` under `// --- Shared Constants ---`.
- Grouped by domain with line-comment headers: `// Window`, `// Paddle`, etc.
- Explicit type annotations: `pub const WINDOW_WIDTH: f32 = 900.0;`
- Colors use `Color::srgb(r, g, b)`.

### Types and Derives

- **Marker components**: unit structs with `#[derive(Component)]` only.
- **Data components**: named fields, all `pub`, with `#[derive(Component)]`.
- **Resources**: `#[derive(Resource)]` with manual `impl Default` for custom defaults.
- **Game state enum**: `#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]`.
- **Custom materials**: `#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]`.
- **Plugins**: plain struct, manual `impl Plugin` — no derives needed.
- Only derive what is actually used — minimal derive sets.

### Error Handling

- **No `unwrap()`, `expect()`, or `panic!()`** — never use these.
- **`let Ok(...) = query.single_mut() else { return; }`** for fallible singleton queries.
- **`if let Ok(...)`** for less critical query results.
- **`if let Some(...)`** for Option values (collision results).
- **`saturating_sub`** for safe decrement without underflow.
- **Early return** with guard clauses: `if !resource.is_changed() { return; }`.
- Systems return `()` — no `Result` return types.

### Comments

- **Doc comments (`///`)** on every `pub fn`: single-line, starts with verb.
  Example: `/// Moves the paddle left/right based on keyboard input.`
- **Section headers** in `components.rs`: `// --- Section Name ---` with triple dashes.
- **Inline comments** above code blocks: `// Paddle`, `// Ball (starts just above paddle)`.
- **No module-level `//!` doc comments**.

### Bevy-Specific Patterns

**System parameter order**:
1. Input resources (`Res<ButtonInput<KeyCode>>`, `Res<Time>`)
2. Mutable state (`ResMut<NextState<...>>`, `Commands`)
3. Queries (`Query<...>`)

Exception: `Commands` comes first in spawn-focused systems.

**App builder order in `main.rs`**:
1. `add_plugins(DefaultPlugins.set(...))` then custom plugins
2. `.init_state::<T>()` — with `// State` comment
3. `.init_resource::<T>()` — with `// Resources` comment
4. `.add_systems(Startup, ...)` — with `// Startup systems` comment
5. State-grouped systems — each group labeled `// Menu state`, `// Playing state`, etc.

**Entity spawning**: tuple bundles `commands.spawn((Component, Component, ...))`.
**Sprite sizing**: `Sprite { custom_size: Some(Vec2::new(...)), ..default() }`.
**z-ordering**: background `-100.0`, default entities `0.0`, ball `1.0`.
**Singleton queries**: `query.single_mut()` with `let ... else` guard.

**Plugins** are self-contained: own their types, startup systems, and update systems.
Plugin-internal functions are **private**; cross-module functions are `pub`.

## Git Conventions

- **Default branch**: `master`
- **Feature branches**: `feat/<feature-name>` (kebab-case)
- **Commit format**: `type: short description` (lowercase, imperative, no period)
  - Body separated by blank line, uses `-` bullet lists
  - Types: `feat:` for new features, `fix:` for bug fixes, `refactor:`, `docs:`, `chore:`
- **Workflow**: feature branch -> merge to `master` when working
- **Dependencies**: always add via `cargo add`, never edit Cargo.toml manually

## Architecture Notes

- `components.rs` is the shared "prelude" — holds all types other modules need.
- Collision uses AABB (axis-aligned bounding box) via `check_aabb_collision()` — reused
  by wall, paddle, and brick collision systems (DRY).
- Game states: `Menu -> Playing -> GameOver | Victory -> Menu` (via SPACE key).
- No `unsafe`, no `async`, no logging/tracing instrumentation.
