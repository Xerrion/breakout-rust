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
