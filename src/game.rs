use bevy::app::AppExit;
use bevy::ecs::prelude::MessageWriter;
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

/// Spawns the pause menu with Resume and Quit buttons.
pub fn spawn_pause_overlay(mut commands: Commands, mut menu_state: ResMut<PauseMenuState>) {
    // Reset menu selection to Resume
    menu_state.selected = 0;

    // Semi-transparent full-screen background (z-index via spawn order)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GlobalZIndex(10),
        OverlayUi,
    ));

    // Menu container (centered, in front of background)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            GlobalZIndex(11),
            OverlayUi,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Resume button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_HOVERED), // Selected by default
                    ResumeButton,
                ))
                .with_child((
                    Text::new("Resume"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

            // Quit button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_NORMAL),
                    QuitButton,
                ))
                .with_child((
                    Text::new("Quit"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

/// Handles mouse interaction with pause menu buttons.
#[allow(clippy::type_complexity)]
pub fn pause_menu_mouse_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ResumeButton>,
            Option<&QuitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
    mut menu_state: ResMut<PauseMenuState>,
) {
    for (interaction, mut bg_color, is_resume, is_quit) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BUTTON_PRESSED.into();
                if is_resume.is_some() {
                    next_state.set(GameState::Playing);
                } else if is_quit.is_some() {
                    app_exit.write(AppExit::Success);
                }
            }
            Interaction::Hovered => {
                *bg_color = BUTTON_HOVERED.into();
                // Update keyboard selection to match hovered button
                if is_resume.is_some() {
                    menu_state.selected = 0;
                } else if is_quit.is_some() {
                    menu_state.selected = 1;
                }
            }
            Interaction::None => {
                // Only reset to normal if not currently keyboard-selected
                let is_selected = (is_resume.is_some() && menu_state.selected == 0)
                    || (is_quit.is_some() && menu_state.selected == 1);
                if !is_selected {
                    *bg_color = BUTTON_NORMAL.into();
                }
            }
        }
    }
}

/// Handles keyboard navigation in the pause menu.
pub fn pause_menu_keyboard_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<PauseMenuState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    // Navigate up/down
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        menu_state.selected = menu_state.selected.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        menu_state.selected = (menu_state.selected + 1).min(PAUSE_MENU_ITEMS - 1);
    }

    // Activate selected button
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        match menu_state.selected {
            0 => next_state.set(GameState::Playing), // Resume
            1 => {
                app_exit.write(AppExit::Success); // Quit
            }
            _ => {}
        }
    }
}

/// Updates button visuals based on keyboard selection state.
#[allow(clippy::type_complexity)]
pub fn update_pause_menu_visuals(
    menu_state: Res<PauseMenuState>,
    mut resume_query: Query<(&mut BackgroundColor, &Interaction), With<ResumeButton>>,
    mut quit_query: Query<
        (&mut BackgroundColor, &Interaction),
        (With<QuitButton>, Without<ResumeButton>),
    >,
) {
    if !menu_state.is_changed() {
        return;
    }

    // Update Resume button
    if let Ok((mut bg_color, interaction)) = resume_query.single_mut()
        && *interaction != Interaction::Hovered
        && *interaction != Interaction::Pressed
    {
        *bg_color = if menu_state.selected == 0 {
            BUTTON_HOVERED.into()
        } else {
            BUTTON_NORMAL.into()
        };
    }

    // Update Quit button
    if let Ok((mut bg_color, interaction)) = quit_query.single_mut()
        && *interaction != Interaction::Hovered
        && *interaction != Interaction::Pressed
    {
        *bg_color = if menu_state.selected == 1 {
            BUTTON_HOVERED.into()
        } else {
            BUTTON_NORMAL.into()
        };
    }
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

    // --- pause_menu_keyboard_navigation ---

    fn pause_menu_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<GameState>();
        app.init_resource::<PauseMenuState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app
    }

    #[test]
    fn keyboard_navigation_moves_selection_down() {
        let mut app = pause_menu_test_app();
        app.add_systems(Update, pause_menu_keyboard_navigation);

        // Start with Resume selected (default = 0)
        assert_eq!(app.world().resource::<PauseMenuState>().selected, 0);

        // Press down arrow
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        app.update();

        assert_eq!(
            app.world().resource::<PauseMenuState>().selected,
            1,
            "Down arrow should move selection to Quit"
        );
    }

    #[test]
    fn keyboard_navigation_moves_selection_up() {
        let mut app = pause_menu_test_app();
        app.add_systems(Update, pause_menu_keyboard_navigation);

        // Start with Quit selected
        app.world_mut().resource_mut::<PauseMenuState>().selected = 1;

        // Press up arrow
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);
        app.update();

        assert_eq!(
            app.world().resource::<PauseMenuState>().selected,
            0,
            "Up arrow should move selection to Resume"
        );
    }

    #[test]
    fn keyboard_navigation_clamps_at_top() {
        let mut app = pause_menu_test_app();
        app.add_systems(Update, pause_menu_keyboard_navigation);

        // Start at top (Resume)
        assert_eq!(app.world().resource::<PauseMenuState>().selected, 0);

        // Press up arrow - should stay at 0
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);
        app.update();

        assert_eq!(
            app.world().resource::<PauseMenuState>().selected,
            0,
            "Selection should not go below 0"
        );
    }

    #[test]
    fn keyboard_navigation_clamps_at_bottom() {
        let mut app = pause_menu_test_app();
        app.add_systems(Update, pause_menu_keyboard_navigation);

        // Start at bottom (Quit)
        app.world_mut().resource_mut::<PauseMenuState>().selected = 1;

        // Press down arrow - should stay at 1
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        app.update();

        assert_eq!(
            app.world().resource::<PauseMenuState>().selected,
            1,
            "Selection should not exceed menu items"
        );
    }

    #[test]
    fn enter_on_resume_transitions_to_playing() {
        let mut app = pause_menu_test_app();
        app.add_systems(Update, pause_menu_keyboard_navigation);

        // Set state to Paused
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update();

        // Resume is selected by default (0)
        // Press Enter
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();
        app.update(); // Apply state transition

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(
            *state.get(),
            GameState::Playing,
            "Enter on Resume should transition to Playing"
        );
    }

    #[test]
    fn space_on_resume_transitions_to_playing() {
        let mut app = pause_menu_test_app();
        app.add_systems(Update, pause_menu_keyboard_navigation);

        // Set state to Paused
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update();

        // Resume is selected by default (0)
        // Press Space
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        app.update(); // Apply state transition

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(
            *state.get(),
            GameState::Playing,
            "Space on Resume should transition to Playing"
        );
    }
}
