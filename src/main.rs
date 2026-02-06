mod background;
mod collision;
mod components;
mod game;
mod movement;
mod setup;

use bevy::prelude::*;
use components::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Breakout".to_string(),
                resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(background::BackgroundPlugin)
        // State
        .init_state::<GameState>()
        // Resources
        .init_resource::<Scoreboard>()
        .init_resource::<Lives>()
        .init_resource::<PauseMenuState>()
        // Startup systems
        .add_systems(
            Startup,
            (setup::spawn_camera, setup::spawn_game, setup::spawn_ui),
        )
        // Menu state
        .add_systems(OnEnter(GameState::Menu), setup::spawn_menu)
        .add_systems(OnExit(GameState::Menu), setup::despawn_overlay)
        .add_systems(Update, game::menu_input.run_if(in_state(GameState::Menu)))
        // Playing state
        .add_systems(OnEnter(GameState::Playing), setup::reset_ball_and_paddle)
        .add_systems(
            Update,
            (
                movement::move_paddle,
                movement::move_ball,
                collision::ball_collision_walls_and_paddle,
                collision::ball_collision_bricks,
                collision::clamp_ball_to_bounds,
                collision::ball_death_zone,
                game::update_scoreboard_ui,
                game::update_lives_ui,
                game::check_game_over,
                game::check_victory,
            )
                .chain()
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
            game::pause_input.run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
        )
        // GameOver / Victory
        .add_systems(OnExit(GameState::GameOver), setup::despawn_overlay)
        .add_systems(OnExit(GameState::Victory), setup::despawn_overlay)
        .add_systems(
            Update,
            game::restart_input
                .run_if(in_state(GameState::GameOver).or(in_state(GameState::Victory))),
        )
        .add_systems(OnEnter(GameState::Menu), game::respawn_on_menu_enter)
        .run();
}
