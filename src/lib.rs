pub mod background;
pub mod collision;
pub mod components;
pub mod environment;
pub mod game;
pub mod gameplay;
pub mod movement;
pub mod presentation;
pub mod setup;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use components::*;
use gameplay::GameplayPlugin;
use presentation::PresentationPlugin;

/// Builds the rendered game app (window, rendering, UI, audio and input).
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
    .add_plugins((GameplayPlugin, PresentationPlugin));
    app
}

/// Runs the rendered game.
pub fn run() {
    game_app().run();
}
