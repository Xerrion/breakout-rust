use bevy::prelude::*;

use crate::components::*;

/// Moves the paddle left/right based on keyboard input, clamped to window bounds.
pub fn move_paddle(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Paddle>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut direction = 0.0;

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }

    transform.translation.x += direction * PADDLE_SPEED * time.delta_secs();

    // Clamp within window bounds
    let max_x = WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;
    transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
}

/// Moves the ball by its velocity each frame.
pub fn move_ball(time: Res<Time>, mut query: Query<(&mut Transform, &Ball)>) {
    for (mut transform, ball) in &mut query {
        transform.translation.x += ball.velocity.x * time.delta_secs();
        transform.translation.y += ball.velocity.y * time.delta_secs();
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

    // --- move_ball ---

    #[test]
    fn ball_moves_in_velocity_direction() {
        let mut app = test_app();
        app.add_systems(Update, move_ball);

        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            Ball {
                velocity: Vec2::new(100.0, 200.0),
            },
        ));

        // First update initializes Time, second update has a real delta
        app.update();
        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Ball)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(transform.translation.x > 0.0, "Ball should move right");
        assert!(transform.translation.y > 0.0, "Ball should move up");
    }

    // --- move_paddle ---

    #[test]
    fn paddle_stays_without_input() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(100.0, PADDLE_Y, 0.0), Paddle));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            (transform.translation.x - 100.0).abs() < 0.01,
            "Paddle should not move without input"
        );
    }

    #[test]
    fn paddle_clamps_to_right_bound() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, move_paddle);

        let max_x = WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;

        // Place paddle beyond right bound
        app.world_mut()
            .spawn((Transform::from_xyz(max_x + 100.0, PADDLE_Y, 0.0), Paddle));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            transform.translation.x <= max_x + 0.01,
            "Paddle should be clamped to right bound, got x={}",
            transform.translation.x
        );
    }

    #[test]
    fn paddle_clamps_to_left_bound() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, move_paddle);

        let max_x = WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;

        // Place paddle beyond left bound
        app.world_mut()
            .spawn((Transform::from_xyz(-max_x - 100.0, PADDLE_Y, 0.0), Paddle));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            transform.translation.x >= -max_x - 0.01,
            "Paddle should be clamped to left bound, got x={}",
            transform.translation.x
        );
    }
}
