use bevy::prelude::*;

use crate::components::*;

/// Updates the score text when the score changes.
pub fn update_scoreboard_ui(
    scoreboard: Res<Scoreboard>,
    mut query: Query<&mut Text, With<ScoreboardUi>>,
) {
    if !scoreboard.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Score: {}", scoreboard.score);
    }
}

/// Updates the lives text when lives change.
pub fn update_lives_ui(lives: Res<Lives>, mut query: Query<&mut Text, With<LivesUi>>) {
    if !lives.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Lives: {}", lives.count);
    }
}

/// Transitions to GameOver when lives reach 0.
pub fn check_game_over(
    lives: Res<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if lives.count == 0 {
        next_state.set(GameState::GameOver);
        commands.spawn((
            Text::new("GAME OVER\n\nPress SPACE to restart"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.3, 0.3)),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(35.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            OverlayUi,
        ));
    }
}

/// Transitions to Victory when all bricks are destroyed.
pub fn check_victory(
    brick_query: Query<&Brick>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    scoreboard: Res<Scoreboard>,
) {
    if brick_query.is_empty() {
        next_state.set(GameState::Victory);
        commands.spawn((
            Text::new(format!(
                "YOU WIN!\n\nScore: {}\n\nPress SPACE to restart",
                scoreboard.score
            )),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.3, 1.0, 0.3)),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            OverlayUi,
        ));
    }
}

/// Handles SPACE press on the menu screen to start the game.
pub fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

/// Handles SPACE press on GameOver/Victory screens to restart.
#[allow(clippy::too_many_arguments)]
pub fn restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut lives: ResMut<Lives>,
    brick_query: Query<Entity, With<Brick>>,
    ball_query: Query<Entity, With<Ball>>,
    paddle_query: Query<Entity, With<Paddle>>,
    wall_query: Query<Entity, With<Wall>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Reset resources
        scoreboard.score = 0;
        lives.count = 3;

        // Despawn all game entities
        for entity in brick_query
            .iter()
            .chain(ball_query.iter())
            .chain(paddle_query.iter())
            .chain(wall_query.iter())
        {
            commands.entity(entity).despawn();
        }

        // Re-spawn the game
        next_state.set(GameState::Menu);
    }
}

/// Re-spawns game entities when entering Menu (after a restart).
pub fn respawn_on_menu_enter(
    commands: Commands,
    paddle_query: Query<&Paddle>,
    mut first_run: Local<bool>,
) {
    // Skip on first run — entities already spawned by Startup, but commands
    // haven't been applied yet so the query would be empty.
    if !*first_run {
        *first_run = true;
        return;
    }

    // Only respawn if there's no paddle (i.e., coming from a restart)
    if paddle_query.is_empty() {
        crate::setup::spawn_game(commands);
    }
}

/// Toggles pause when ESC is pressed during gameplay.
pub fn pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

/// Spawns the pause overlay with semi-transparent background.
pub fn spawn_pause_overlay(mut commands: Commands) {
    // Semi-transparent full-screen background
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        OverlayUi,
    ));

    // Pause text
    commands.spawn((
        Text::new("PAUSED\n\nPress ESC to resume"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        OverlayUi,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<GameState>();
        app.init_resource::<Scoreboard>();
        app.init_resource::<Lives>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app
    }

    // --- check_game_over ---

    #[test]
    fn game_over_when_no_lives() {
        let mut app = test_app();
        app.add_systems(Update, check_game_over);
        app.world_mut().resource_mut::<Lives>().count = 0;

        app.update();

        let mut q = app.world_mut().query::<&OverlayUi>();
        let overlay_count = q.iter(app.world()).count();
        assert_eq!(overlay_count, 1, "Should spawn a game-over overlay");
    }

    #[test]
    fn no_game_over_with_lives_remaining() {
        let mut app = test_app();
        app.add_systems(Update, check_game_over);

        app.update();

        let mut q = app.world_mut().query::<&OverlayUi>();
        let overlay_count = q.iter(app.world()).count();
        assert_eq!(overlay_count, 0, "Should not spawn overlay when lives > 0");
    }

    // --- check_victory ---

    #[test]
    fn victory_when_no_bricks() {
        let mut app = test_app();
        app.add_systems(Update, check_victory);
        // No bricks spawned

        app.update();

        let mut q = app.world_mut().query::<&OverlayUi>();
        let overlay_count = q.iter(app.world()).count();
        assert_eq!(overlay_count, 1, "Should spawn a victory overlay");
    }

    #[test]
    fn no_victory_with_bricks_remaining() {
        let mut app = test_app();
        app.add_systems(Update, check_victory);

        // Spawn a brick
        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 100.0, 0.0), Brick));

        app.update();

        let mut q = app.world_mut().query::<&OverlayUi>();
        let overlay_count = q.iter(app.world()).count();
        assert_eq!(
            overlay_count, 0,
            "Should not spawn overlay when bricks remain"
        );
    }

    // --- update_scoreboard_ui ---

    #[test]
    fn scoreboard_ui_updates_on_change() {
        let mut app = test_app();
        app.add_systems(Update, update_scoreboard_ui);

        app.world_mut().spawn((Text::new("Score: 0"), ScoreboardUi));

        app.world_mut().resource_mut::<Scoreboard>().score = 42;

        app.update();

        let mut q = app.world_mut().query::<(&Text, &ScoreboardUi)>();
        let text = q.iter(app.world()).next().unwrap().0;
        assert_eq!(**text, "Score: 42");
    }

    // --- pause_input ---

    #[test]
    fn pause_input_pauses_game() {
        let mut app = test_app();
        app.add_systems(Update, pause_input);

        // Set state to Playing
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        // Simulate ESC press - press key, then update
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        // State transitions are applied in StateTransition schedule,
        // need another update for the state to actually change
        app.update();

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(
            *state.get(),
            GameState::Paused,
            "ESC in Playing should transition to Paused"
        );
    }

    #[test]
    fn pause_input_resumes_game() {
        let mut app = test_app();
        app.add_systems(Update, pause_input);

        // Set state to Paused
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update();

        // Simulate ESC press - press key, then update
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        // State transitions are applied in StateTransition schedule,
        // need another update for the state to actually change
        app.update();

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(
            *state.get(),
            GameState::Playing,
            "ESC in Paused should transition to Playing"
        );
    }

    #[test]
    fn pause_input_ignored_in_menu() {
        let mut app = test_app();
        app.add_systems(Update, pause_input);

        // State starts in Menu (default)
        app.update();

        // Simulate ESC press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(
            *state.get(),
            GameState::Menu,
            "ESC in Menu should not change state"
        );
    }

    #[test]
    fn pause_input_ignored_in_game_over() {
        let mut app = test_app();
        app.add_systems(Update, pause_input);

        // Set state to GameOver
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::GameOver);
        app.update();

        // Simulate ESC press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(
            *state.get(),
            GameState::GameOver,
            "ESC in GameOver should not change state"
        );
    }
}
