# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Reference Project

`docs/suika-game/` contains a complete, working Bevy game. **Always read the relevant file there before implementing any new feature** — it shows real, compiling Bevy 0.17 patterns for the same crate structure.

| What you're implementing | Reference file(s) |
|---|---|
| Camera (follow / setup) | `app/ui/src/camera.rs` |
| UI screens (title, pause, game-over) | `app/ui/src/screens/{title,pause,game_over}.rs` |
| HUD elements (score, timer, next-item) | `app/ui/src/screens/hud/` |
| UI node styles / colours | `app/ui/src/styles.rs` |
| UI components / markers | `app/ui/src/components.rs` |
| Entity spawn system | `app/core/src/systems/spawn.rs` |
| Input handling | `app/core/src/systems/input.rs` |
| Collision detection | `app/core/src/systems/collision.rs` |
| Game-over / reset logic | `app/core/src/systems/game_over.rs` |
| Pause / resume | `app/core/src/systems/pause.rs` |
| Score / XP accumulation | `app/core/src/systems/score.rs` |
| Visual effects (flash, shake, particles) | `app/core/src/systems/effects/` |
| Events (custom Bevy events) | `app/core/src/events.rs` |
| Resources (split into sub-modules) | `app/core/src/resources/` |
| Plugin wiring (`lib.rs`) | `app/core/src/lib.rs` |
| Persistence (save/load JSON) | `app/core/src/persistence.rs` |
| Asset loading plugin | `app/assets/src/lib.rs` |
| Audio (BGM + SFX channels) | `app/audio/src/{bgm,channels,handles}.rs`, `app/audio/src/sfx/` |
| RON config / hot-reload | `app/core/src/config/` |

## Commands

```bash
just run          # Run the game (debug)
just dev          # Run with RUST_LOG=debug (gameplay logging)
just build        # Build workspace (debug)
just test         # Run all tests
just check        # fmt --check + clippy -D warnings
just fmt          # Auto-format all code
just clippy       # Clippy -D warnings

# Targeted test runs
just unit-test vs-core                      # All unit tests in vs-core
just unit-test vs-core spatial_grid_clear  # Single test by name
cargo test -p vs-core -- --nocapture       # Tests with stdout
```

## Crate Architecture

Five-crate workspace under `app/`:

| Crate | Path | Purpose |
|---|---|---|
| `vs-core` | `app/core/` | All game logic, ECS components/resources/systems |
| `vs-ui` | `app/ui/` | Camera, UI screens, HUD (depends on vs-core) |
| `vs-audio` | `app/audio/` | BGM/SFX via bevy_kira_audio |
| `vs-assets` | `app/assets/` | Sprite/font/audio asset loading |
| `vs` | `app/vampire-survivors/` | Binary: wires the four plugins together |

Assets live in `app/vampire-survivors/assets/` (Bevy resolves paths relative to the binary crate).

## Game State Machine

```
Title → CharacterSelect → Playing ←─── LevelUp (returns after choice)
  │                          │  ↕ ESC
  └──→ MetaShop              │  Paused → Playing (resume) / Title (quit)
                             ├──→ GameOver  (HP = 0)
                             └──→ Victory   (boss defeated)
```

Default state is `Title`. To test `Playing`-state systems during development, temporarily change `#[default]` to `Playing` in `states.rs`.

## vs-core Module Layout

- `states.rs` — `AppState` enum
- `types.rs` — domain enums (`WeaponType`, `EnemyType`, `CharacterType`, etc.) — no Bevy deps
- `constants.rs` — all numeric constants; `PlayerStats::default()` reads from here
- `components.rs` — all ECS components (`Player`, `Enemy`, `Projectile`, `CircleCollider`, …)
- `resources.rs` — `GameData`, `SpatialGrid`, `MetaProgress`, spawner resources
- `player.rs` — `spawn_player` (OnEnter Playing) + `player_movement` (Update)
- `lib.rs` — `GameCorePlugin`: registers state, all resources, and wires systems

## Collision Detection

No physics engine. Collision uses a manual `SpatialGrid` (64 px cells, `HashMap<(i32,i32), Vec<Entity>>`). Each frame: `clear()` → insert all entities → `get_nearby(pos, radius)` → exact distance check. Returns potential false positives; callers must verify distance.

## Entity Lifecycle

Use `DespawnOnExit(AppState::Playing)` (from `bevy::state::state_scoped`) on entities spawned in `OnEnter(Playing)` — they are cleaned up automatically. **Note:** `StateScoped<S>` is a type alias for `DespawnOnExit<S>` in Bevy 0.17 and cannot be used as a constructor; import and use `DespawnOnExit` directly.

## Bevy 0.17 API Notes

- `Query::get_single_mut()` → `Query::single_mut()` (returns `Result`)
- `query_filtered::<D, F>()` requires `&mut World` — split the borrow:
  ```rust
  let mut q = app.world_mut().query_filtered::<Entity, With<Foo>>();
  let entity = q.single(app.world()).unwrap();
  ```
- In tests, `Time::delta_secs()` is 0 after `app.update()` because `TimePlugin` in `First` reads the OS clock. Use `World::run_system_once` + `Time::advance_by` to test movement systems:
  ```rust
  use bevy::ecs::system::RunSystemOnce as _; // in #[cfg(test)] only
  app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(1.0 / 60.0));
  app.world_mut().run_system_once(my_system).unwrap();
  ```
- Integration tests use `MinimalPlugins + bevy::state::app::StatesPlugin`, not `DefaultPlugins`.

## Key Design Constants

| Constant | Value | Notes |
|---|---|---|
| `PLAYER_BASE_SPEED` | 200.0 px/s | Base movement speed |
| `PLAYER_BASE_HP` | 100.0 | |
| `PLAYER_PICKUP_RADIUS` | 80.0 px | XP gem magnet range |
| `SPATIAL_GRID_CELL_SIZE` | 64.0 px | Collision grid cell size |
| `BOSS_SPAWN_TIME` | 1800 s | 30 minutes |
| `XP_LEVEL_BASE` | 20 | XP for first level-up |
| `MAX_WEAPONS` / `MAX_PASSIVES` | 6 / 6 | Inventory caps |
