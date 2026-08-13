use bevy::prelude::*;

use crate::components::*;

/// Translates keyboard input into a `PaddleAction`.
pub fn paddle_input(keyboard: Res<ButtonInput<KeyCode>>, mut action: ResMut<PaddleAction>) {
    let left = keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA);
    let right = keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD);

    *action = match (left, right) {
        (true, false) => PaddleAction::Left,
        (false, true) => PaddleAction::Right,
        _ => PaddleAction::Stay,
    };
}

/// Moves the paddle from the current `PaddleAction`, clamped to window bounds.
pub fn move_paddle(action: Res<PaddleAction>, mut query: Query<&mut Transform, With<Paddle>>) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    transform.translation.x += action.direction() * PADDLE_SPEED * SIM_DT;

    // Clamp within window bounds
    let max_x = WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;
    transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
}

/// Moves the ball by its velocity for one fixed simulation tick.
pub fn move_ball(mut query: Query<(&mut Transform, &Ball)>) {
    for (mut transform, ball) in &mut query {
        transform.translation.x += ball.velocity.x * SIM_DT;
        transform.translation.y += ball.velocity.y * SIM_DT;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<PaddleAction>();
        app
    }

    // --- move_ball ---

    #[test]
    fn ball_moves_exactly_one_fixed_step() {
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
        assert!((transform.translation.x - 100.0 * SIM_DT).abs() < 0.001);
        assert!((transform.translation.y - 200.0 * SIM_DT).abs() < 0.001);
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
        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Ball)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            (transform.translation.x - 2.0 * 100.0 * SIM_DT).abs() < 0.001,
            "Two ticks should always move the same distance"
        );
    }

    // --- paddle_input ---

    #[test]
    fn keyboard_left_sets_left_action() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, paddle_input);

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
        app.add_systems(Update, paddle_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        app.update();

        assert_eq!(*app.world().resource::<PaddleAction>(), PaddleAction::Right);
    }

    #[test]
    fn keyboard_without_input_sets_stay_action() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, paddle_input);

        *app.world_mut().resource_mut::<PaddleAction>() = PaddleAction::Left;
        app.update();

        assert_eq!(*app.world().resource::<PaddleAction>(), PaddleAction::Stay);
    }

    #[test]
    fn keyboard_both_directions_sets_stay_action() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, paddle_input);

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::ArrowLeft);
            input.press(KeyCode::ArrowRight);
        }
        app.update();

        assert_eq!(*app.world().resource::<PaddleAction>(), PaddleAction::Stay);
    }

    // --- move_paddle ---

    #[test]
    fn paddle_does_not_read_keyboard_directly() {
        let mut app = test_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, PADDLE_Y, 0.0), Paddle));

        // Keyboard says left, but no action was written
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            transform.translation.x.abs() < 0.001,
            "Paddle should only react to PaddleAction"
        );
    }

    #[test]
    fn paddle_moves_left_on_left_action() {
        let mut app = test_app();
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, PADDLE_Y, 0.0), Paddle));
        *app.world_mut().resource_mut::<PaddleAction>() = PaddleAction::Left;

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            (transform.translation.x + PADDLE_SPEED * SIM_DT).abs() < 0.001,
            "Paddle should move one fixed step left"
        );
    }

    #[test]
    fn paddle_moves_right_on_right_action() {
        let mut app = test_app();
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, PADDLE_Y, 0.0), Paddle));
        *app.world_mut().resource_mut::<PaddleAction>() = PaddleAction::Right;

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            (transform.translation.x - PADDLE_SPEED * SIM_DT).abs() < 0.001,
            "Paddle should move one fixed step right"
        );
    }

    #[test]
    fn paddle_stays_without_action() {
        let mut app = test_app();
        app.add_systems(Update, move_paddle);

        app.world_mut()
            .spawn((Transform::from_xyz(100.0, PADDLE_Y, 0.0), Paddle));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &Paddle)>();
        let transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            (transform.translation.x - 100.0).abs() < 0.01,
            "Paddle should not move without an action"
        );
    }

    #[test]
    fn paddle_clamps_to_right_bound() {
        let mut app = test_app();
        app.add_systems(Update, move_paddle);

        let max_x = WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;

        app.world_mut()
            .spawn((Transform::from_xyz(max_x + 100.0, PADDLE_Y, 0.0), Paddle));
        *app.world_mut().resource_mut::<PaddleAction>() = PaddleAction::Right;

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
        app.add_systems(Update, move_paddle);

        let max_x = WINDOW_WIDTH / 2.0 - PADDLE_WIDTH / 2.0;

        app.world_mut()
            .spawn((Transform::from_xyz(-max_x - 100.0, PADDLE_Y, 0.0), Paddle));
        *app.world_mut().resource_mut::<PaddleAction>() = PaddleAction::Left;

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
