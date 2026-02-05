use bevy::prelude::*;

use crate::components::*;

/// Spawns the camera.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawns the paddle, ball, bricks, and walls.
pub fn spawn_game(mut commands: Commands) {
    // Paddle
    commands.spawn((
        Sprite {
            color: PADDLE_COLOR,
            custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, PADDLE_Y, 0.0),
        Paddle,
        Collider,
    ));

    // Ball (starts just above paddle)
    let ball_start_y = PADDLE_Y + PADDLE_HEIGHT / 2.0 + BALL_SIZE / 2.0 + 1.0;
    commands.spawn((
        Sprite {
            color: BALL_COLOR,
            custom_size: Some(Vec2::splat(BALL_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, ball_start_y, 1.0),
        Ball {
            velocity: Vec2::new(BALL_SPEED * 0.7, BALL_SPEED),
        },
    ));

    // Bricks
    let grid_width = BRICK_COLS as f32 * (BRICK_WIDTH + BRICK_GAP) - BRICK_GAP;
    let grid_start_x = -grid_width / 2.0 + BRICK_WIDTH / 2.0;
    let grid_start_y = WINDOW_HEIGHT / 2.0 - 80.0;

    for (row, &color) in BRICK_COLORS.iter().enumerate().take(BRICK_ROWS) {
        for col in 0..BRICK_COLS {
            let x = grid_start_x + col as f32 * (BRICK_WIDTH + BRICK_GAP);
            let y = grid_start_y - row as f32 * (BRICK_HEIGHT + BRICK_GAP);

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                    ..default()
                },
                Transform::from_xyz(x, y, 0.0),
                Brick,
                Collider,
            ));
        }
    }

    // Walls (top, left, right — bottom is the death zone)
    let half_w = WINDOW_WIDTH / 2.0;
    let half_h = WINDOW_HEIGHT / 2.0;

    // Top wall
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(
                WINDOW_WIDTH + WALL_THICKNESS * 2.0,
                WALL_THICKNESS,
            )),
            ..default()
        },
        Transform::from_xyz(0.0, half_h + WALL_THICKNESS / 2.0, 0.0),
        Wall,
        Collider,
    ));

    // Left wall
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(
                WALL_THICKNESS,
                WINDOW_HEIGHT + WALL_THICKNESS * 2.0,
            )),
            ..default()
        },
        Transform::from_xyz(-half_w - WALL_THICKNESS / 2.0, 0.0, 0.0),
        Wall,
        Collider,
    ));

    // Right wall
    commands.spawn((
        Sprite {
            color: WALL_COLOR,
            custom_size: Some(Vec2::new(
                WALL_THICKNESS,
                WINDOW_HEIGHT + WALL_THICKNESS * 2.0,
            )),
            ..default()
        },
        Transform::from_xyz(half_w + WALL_THICKNESS / 2.0, 0.0, 0.0),
        Wall,
        Collider,
    ));
}

/// Spawns the HUD: score (top-left) and lives (top-right).
pub fn spawn_ui(mut commands: Commands) {
    // Score text
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreboardUi,
    ));

    // Lives text
    commands.spawn((
        Text::new("Lives: 3"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        LivesUi,
    ));
}

/// Spawns the menu overlay text.
pub fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        Text::new("BREAKOUT\n\nPress SPACE to start"),
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

/// Removes the overlay UI (used on state transitions).
pub fn despawn_overlay(mut commands: Commands, query: Query<Entity, With<OverlayUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Resets ball and paddle positions when entering Playing state.
pub fn reset_ball_and_paddle(
    mut paddle_query: Query<&mut Transform, With<Paddle>>,
    mut ball_query: Query<(&mut Transform, &mut Ball), Without<Paddle>>,
) {
    if let Ok(mut paddle_transform) = paddle_query.single_mut() {
        paddle_transform.translation.x = 0.0;
    }

    if let Ok((mut ball_transform, mut ball)) = ball_query.single_mut() {
        ball_transform.translation.x = 0.0;
        ball_transform.translation.y = PADDLE_Y + PADDLE_HEIGHT / 2.0 + BALL_SIZE / 2.0 + 1.0;
        ball.velocity = Vec2::new(BALL_SPEED * 0.7, BALL_SPEED);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    // --- spawn_game ---

    #[test]
    fn spawn_game_creates_one_paddle() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_game);
        app.update();

        let mut q = app.world_mut().query::<&Paddle>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "Should spawn exactly one paddle");
    }

    #[test]
    fn spawn_game_creates_one_ball() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_game);
        app.update();

        let mut q = app.world_mut().query::<&Ball>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "Should spawn exactly one ball");
    }

    #[test]
    fn spawn_game_creates_fifty_bricks() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_game);
        app.update();

        let mut q = app.world_mut().query::<&Brick>();
        let count = q.iter(app.world()).count();
        assert_eq!(
            count,
            BRICK_ROWS * BRICK_COLS,
            "Should spawn BRICK_ROWS * BRICK_COLS bricks"
        );
    }

    #[test]
    fn spawn_game_creates_three_walls() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_game);
        app.update();

        let mut q = app.world_mut().query::<&Wall>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 3, "Should spawn 3 walls (top, left, right)");
    }

    #[test]
    fn spawn_game_entities_have_colliders() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_game);
        app.update();

        let mut q = app.world_mut().query::<(&Paddle, &Collider)>();
        let paddle_colliders = q.iter(app.world()).count();
        assert_eq!(paddle_colliders, 1, "Paddle should have Collider");

        let mut q = app.world_mut().query::<(&Brick, &Collider)>();
        let brick_colliders = q.iter(app.world()).count();
        assert_eq!(
            brick_colliders,
            BRICK_ROWS * BRICK_COLS,
            "All bricks should have Collider"
        );

        let mut q = app.world_mut().query::<(&Wall, &Collider)>();
        let wall_colliders = q.iter(app.world()).count();
        assert_eq!(wall_colliders, 3, "All walls should have Collider");
    }

    // --- despawn_overlay ---

    #[test]
    fn despawn_overlay_removes_overlay_entities() {
        let mut app = test_app();
        app.add_systems(Update, despawn_overlay);

        app.world_mut().spawn(OverlayUi);
        app.world_mut().spawn(OverlayUi);

        let mut q = app.world_mut().query::<&OverlayUi>();
        let before = q.iter(app.world()).count();
        assert_eq!(before, 2);

        app.update();

        let mut q = app.world_mut().query::<&OverlayUi>();
        let after = q.iter(app.world()).count();
        assert_eq!(after, 0, "All OverlayUi entities should be despawned");
    }

    // --- reset_ball_and_paddle ---

    #[test]
    fn reset_ball_and_paddle_resets_positions() {
        let mut app = test_app();
        app.add_systems(Update, reset_ball_and_paddle);

        // Spawn paddle at off-center position
        app.world_mut()
            .spawn((Transform::from_xyz(200.0, PADDLE_Y, 0.0), Paddle));

        // Spawn ball at off-center position with different velocity
        app.world_mut().spawn((
            Transform::from_xyz(100.0, 50.0, 1.0),
            Ball {
                velocity: Vec2::new(-100.0, -200.0),
            },
        ));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let paddle_x = q.iter(app.world()).next().unwrap().0.translation.x;
        assert!((paddle_x).abs() < 0.01, "Paddle x should reset to 0");

        let mut q = app.world_mut().query::<(&Transform, &Ball)>();
        let (ball_transform, ball) = q.iter(app.world()).next().unwrap();
        assert!(
            (ball_transform.translation.x).abs() < 0.01,
            "Ball x should reset to 0"
        );
        assert!(
            ball.velocity.y > 0.0,
            "Ball should be moving upward after reset"
        );
    }
}
