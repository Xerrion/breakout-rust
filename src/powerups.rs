use bevy::prelude::*;
use bevy::time::TimerMode;

use crate::components::*;

/// Moves power-ups downward at constant speed.
pub fn move_powerups(time: Res<Time>, mut query: Query<&mut Transform, With<PowerUp>>) {
    for mut transform in &mut query {
        transform.translation.y -= POWERUP_FALL_SPEED * time.delta_secs();
    }
}

/// Ticks active power-up timers and expires effects when timers complete.
pub fn tick_powerup_timers(
    time: Res<Time>,
    mut active_powerups: ResMut<ActivePowerUps>,
    mut paddle_state: ResMut<PaddleState>,
    mut ball_speed_modifier: ResMut<BallSpeedModifier>,
) {
    let mut expired_types = Vec::new();

    for (power_type, timer) in active_powerups.timers.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            expired_types.push(*power_type);
        }
    }

    for power_type in &expired_types {
        match power_type {
            PowerUpType::WiderPaddle => {
                paddle_state.current_width = PADDLE_WIDTH;
            }
            PowerUpType::SlowBall => {
                ball_speed_modifier.multiplier = 0.0;
            }
            PowerUpType::MultiBall => {}
        }
    }

    active_powerups
        .timers
        .retain(|(_, timer)| !timer.just_finished());
}

/// Despawns power-ups that fall below the death zone.
pub fn despawn_powerups_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<PowerUp>>,
) {
    let death_y = -WINDOW_HEIGHT / 2.0 - POWERUP_SIZE;

    for (entity, transform) in &query {
        if transform.translation.y < death_y {
            commands.entity(entity).despawn();
        }
    }
}

/// Detects collision between power-ups and the paddle, activates effects.
#[allow(clippy::too_many_arguments)]
pub fn powerup_paddle_collision(
    mut commands: Commands,
    powerup_query: Query<(Entity, &Transform, &PowerUp)>,
    paddle_query: Query<&Transform, With<Paddle>>,
    ball_query: Query<(&Transform, &Ball)>,
    mut paddle_state: ResMut<PaddleState>,
    mut ball_speed_modifier: ResMut<BallSpeedModifier>,
    mut active_powerups: ResMut<ActivePowerUps>,
) {
    let Ok(paddle_transform) = paddle_query.single() else {
        return;
    };

    let paddle_pos = paddle_transform.translation.truncate();
    let paddle_size = Vec2::new(paddle_state.current_width, PADDLE_HEIGHT);

    for (powerup_entity, powerup_transform, powerup) in &powerup_query {
        let powerup_pos = powerup_transform.translation.truncate();
        let powerup_size = Vec2::splat(POWERUP_SIZE);

        if check_aabb_collision(powerup_pos, powerup_size, paddle_pos, paddle_size).is_some() {
            match powerup.power_type {
                PowerUpType::MultiBall => {
                    spawn_multi_balls(&mut commands, &ball_query, powerup_pos);
                }
                PowerUpType::WiderPaddle => {
                    paddle_state.current_width = PADDLE_WIDTH * WIDER_PADDLE_MULTIPLIER;
                    reset_or_add_timer(&mut active_powerups, PowerUpType::WiderPaddle);
                }
                PowerUpType::SlowBall => {
                    ball_speed_modifier.multiplier = SLOW_BALL_MULTIPLIER;
                    reset_or_add_timer(&mut active_powerups, PowerUpType::SlowBall);
                }
            }

            commands.entity(powerup_entity).despawn();
        }
    }
}

fn spawn_multi_balls(
    commands: &mut Commands,
    ball_query: &Query<(&Transform, &Ball)>,
    powerup_pos: Vec2,
) {
    let mut closest_ball: Option<(&Transform, &Ball, f32)> = None;

    for (transform, ball) in ball_query.iter() {
        let dist = transform.translation.truncate().distance(powerup_pos);
        match closest_ball {
            None => closest_ball = Some((transform, ball, dist)),
            Some((_, _, prev_dist)) if dist < prev_dist => {
                closest_ball = Some((transform, ball, dist));
            }
            _ => {}
        }
    }

    let Some((ball_transform, ball, _)) = closest_ball else {
        return;
    };

    let base_velocity = ball.velocity;
    let ball_pos = ball_transform.translation;

    for angle_offset in [-std::f32::consts::FRAC_PI_6, std::f32::consts::FRAC_PI_6] {
        let rotated_velocity = rotate_vec2(base_velocity, angle_offset);

        commands.spawn((
            Sprite {
                color: BALL_COLOR,
                custom_size: Some(Vec2::splat(BALL_SIZE)),
                ..default()
            },
            Transform::from_translation(ball_pos),
            Ball {
                velocity: rotated_velocity,
            },
        ));
    }
}

fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn reset_or_add_timer(active_powerups: &mut ActivePowerUps, power_type: PowerUpType) {
    for (existing_type, timer) in active_powerups.timers.iter_mut() {
        if *existing_type == power_type {
            timer.reset();
            return;
        }
    }

    active_powerups.timers.push((
        power_type,
        Timer::from_seconds(POWERUP_DURATION, TimerMode::Once),
    ));
}

/// Spawns a power-up entity at the given position with the specified type.
pub fn spawn_powerup(commands: &mut Commands, position: Vec3, power_type: PowerUpType) {
    let color = match power_type {
        PowerUpType::MultiBall => POWERUP_MULTIBALL_COLOR,
        PowerUpType::WiderPaddle => POWERUP_WIDERPADDLE_COLOR,
        PowerUpType::SlowBall => POWERUP_SLOWBALL_COLOR,
    };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(POWERUP_SIZE)),
            ..default()
        },
        Transform::from_translation(position.truncate().extend(0.5)),
        PowerUp { power_type },
    ));
}

/// Returns a random power-up type.
pub fn random_powerup_type() -> PowerUpType {
    let rand_val = rand::random::<f32>();
    if rand_val < 0.33 {
        PowerUpType::MultiBall
    } else if rand_val < 0.66 {
        PowerUpType::WiderPaddle
    } else {
        PowerUpType::SlowBall
    }
}
