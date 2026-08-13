use bevy::prelude::*;

use crate::components::*;

/// Translates keyboard input into a `PaddleAction`.
pub fn paddle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action: ResMut<PaddleAction>,
) {
    let left = keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA);
    let right = keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD);

    *action = match (left, right) {
        (true, false) => PaddleAction::Left,
        (false, true) => PaddleAction::Right,
        _ => PaddleAction::Stay,
    };
}

/// Moves the paddle according to the current `PaddleAction`, clamped to window bounds.
pub fn move_paddle(action: Res<PaddleAction>, mut query: Query<&mut Transform, With<Paddle>>) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    transform.translation.x += action.direction() * PADDLE_SPEED * FIXED_TIMESTEP;

    // Clamp within window bounds
    let max_x = WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;
    transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
}

/// Moves the ball by its velocity using the fixed simulation timestep.
pub fn move_ball(mut query: Query<(&mut Transform, &Ball)>) {
    for (mut transform, ball) in &mut query {
        transform.translation.x += ball.velocity.x * FIXED_TIMESTEP;
        transform.translation.y += ball.velocity.y * FIXED_TIMESTEP;
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

    // --- paddle_keyboard_input ---

    #[test]
    fn keyboard_left_sets_left_action() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<PaddleAction>();
        app.add_systems(Update, paddle_keyboard_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        app.update();

        assert_eq!(*app.world().resource::<PaddleAction>(), PaddleAction::Left);
    }

    #[test]
    fn keyboard_right_sets_right_action() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<PaddleAction>();
        app.add_systems(Update, paddle_keyboard_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        app.update();

        assert_eq!(*app.world().resource::<PaddleAction>(), PaddleAction::Right);
    }

    #[test]
    fn keyboard_both_directions_sets_stay() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.insert_resource(PaddleAction::Left);
        app.add_systems(Update, paddle_keyboard_input);

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::ArrowLeft);
            input.press(KeyCode::ArrowRight);
        }
        app.update();

        assert_eq!(*app.world().resource::<PaddleAction>(), PaddleAction::Stay);
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

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Ball)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(transform.translation.x > 0.0, "Ball should move right");
        assert!(transform.translation.y > 0.0, "Ball should move up");
    }

    #[test]
    fn ball_movement_is_independent_of_real_time() {
        let mut app = test_app();
        app.add_systems(Update, move_ball);

        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            Ball {
                velocity: Vec2::new(100.0, 0.0),
            },
        ));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Ball)>();
        let x = q.iter(app.world()).next().unwrap().0.translation.x;
        assert!(
            (x - 100.0 * FIXED_TIMESTEP).abs() < 1e-4,
            "Ball should advance exactly one fixed timestep, got x={x}"
        );
    }

    // --- move_paddle ---

    #[test]
    fn paddle_stays_on_stay_action() {
        let mut app = test_app();
        app.init_resource::<PaddleAction>();
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(100.0, PADDLE_Y, 0.0), Paddle));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            (transform.translation.x - 100.0).abs() < 0.01,
            "Paddle should not move on Stay action"
        );
    }

    #[test]
    fn paddle_moves_left_on_left_action() {
        let mut app = test_app();
        app.insert_resource(PaddleAction::Left);
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, PADDLE_Y, 0.0), Paddle));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let x = q.iter(app.world()).next().unwrap().0.translation.x;
        assert!((x + PADDLE_SPEED * FIXED_TIMESTEP).abs() < 1e-4);
    }

    #[test]
    fn paddle_moves_right_on_right_action() {
        let mut app = test_app();
        app.insert_resource(PaddleAction::Right);
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, PADDLE_Y, 0.0), Paddle));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let x = q.iter(app.world()).next().unwrap().0.translation.x;
        assert!((x - PADDLE_SPEED * FIXED_TIMESTEP).abs() < 1e-4);
    }

    #[test]
    fn paddle_clamps_to_right_bound() {
        let mut app = test_app();
        app.init_resource::<PaddleAction>();
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
        app.init_resource::<PaddleAction>();
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
