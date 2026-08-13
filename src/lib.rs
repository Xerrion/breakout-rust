pub mod background;
pub mod collision;
pub mod components;
pub mod environment;
pub mod game;
pub mod gameplay;
pub mod movement;
pub mod setup;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use components::*;
use gameplay::{GameplayPlugin, run_simulation_tick};

/// Rendering, UI and player input on top of the gameplay simulation.
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(background::BackgroundPlugin)
            .init_resource::<PauseMenuState>()
            // Startup systems
            .add_systems(Startup, (setup::spawn_camera, setup::spawn_ui))
            // Menu state
            .add_systems(OnEnter(GameState::Menu), setup::spawn_menu)
            .add_systems(OnExit(GameState::Menu), setup::despawn_overlay)
            .add_systems(Update, game::menu_input.run_if(in_state(GameState::Menu)))
            // Playing state
            .add_systems(
                Update,
                (
                    movement::paddle_input,
                    game::update_scoreboard_ui,
                    game::update_lives_ui,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Paused state
            .add_systems(OnEnter(GameState::Paused), game::spawn_pause_overlay)
            .add_systems(OnExit(GameState::Paused), setup::despawn_overlay)
            .add_systems(
                Update,
                (
                    game::pause_menu_mouse_interaction,
                    game::pause_menu_keyboard_navigation,
                    game::update_pause_menu_visuals,
                )
                    .run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                Update,
                game::pause_input
                    .run_if(in_state(GameState::Playing).or_else(in_state(GameState::Paused))),
            )
            // GameOver / Victory
            .add_systems(OnEnter(GameState::GameOver), setup::spawn_game_over_overlay)
            .add_systems(OnEnter(GameState::Victory), setup::spawn_victory_overlay)
            .add_systems(OnExit(GameState::GameOver), setup::despawn_overlay)
            .add_systems(OnExit(GameState::Victory), setup::despawn_overlay)
            .add_systems(
                Update,
                game::restart_input
                    .run_if(in_state(GameState::GameOver).or_else(in_state(GameState::Victory))),
            )
            .add_systems(OnEnter(GameState::Menu), game::respawn_on_menu_enter);
    }
}

/// Builds the windowed game, driving the simulation at a fixed timestep.
pub fn game_app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Breakout".to_string(),
                    resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            }),
    )
    .add_plugins((GameplayPlugin, PresentationPlugin))
    .insert_resource(Time::<Fixed>::from_seconds(SIM_DT as f64))
    .add_systems(FixedUpdate, run_simulation_tick);

    app
}
