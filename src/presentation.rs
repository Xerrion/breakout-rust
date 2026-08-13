use bevy::prelude::*;

use crate::background::BackgroundPlugin;
use crate::components::*;
use crate::game;
use crate::movement;
use crate::setup;

/// Rendering, UI and player input.
///
/// Everything in this plugin is optional: the gameplay simulation runs without
/// it in headless/training mode.
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BackgroundPlugin)
            .init_resource::<PauseMenuState>()
            // Rendering / HUD setup
            .add_systems(Startup, (setup::spawn_camera, setup::spawn_ui))
            // Player input is read before the fixed-timestep simulation runs
            .add_systems(PreUpdate, movement::paddle_keyboard_input)
            // Menu state
            .add_systems(OnEnter(GameState::Menu), setup::spawn_menu)
            .add_systems(OnExit(GameState::Menu), setup::despawn_overlay)
            .add_systems(Update, game::menu_input.run_if(in_state(GameState::Menu)))
            // HUD
            .add_systems(
                Update,
                (game::update_scoreboard_ui, game::update_lives_ui)
                    .run_if(in_state(GameState::Playing).or_else(in_state(GameState::Paused))),
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
            .add_systems(OnExit(GameState::GameOver), setup::despawn_overlay)
            .add_systems(OnEnter(GameState::Victory), setup::spawn_victory_overlay)
            .add_systems(OnExit(GameState::Victory), setup::despawn_overlay)
            .add_systems(
                Update,
                game::restart_input
                    .run_if(in_state(GameState::GameOver).or_else(in_state(GameState::Victory))),
            );
    }
}
