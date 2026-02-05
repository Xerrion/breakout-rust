use bevy::prelude::*;

use crate::components::*;

/// Ball vs walls and paddle — reflect velocity on collision.
pub fn ball_collision_walls_and_paddle(
    mut ball_query: Query<(&mut Transform, &mut Ball)>,
    collider_query: Query<
        (&Transform, Option<&Paddle>, Option<&Wall>),
        (With<Collider>, Without<Ball>, Without<Brick>),
    >,
) {
    let Ok((mut ball_transform, mut ball)) = ball_query.single_mut() else {
        return;
    };

    let ball_pos = ball_transform.translation.truncate();
    let ball_size = Vec2::splat(BALL_SIZE);

    for (collider_transform, paddle, wall) in &collider_query {
        let target_pos = collider_transform.translation.truncate();
        let target_size = collider_transform.scale.truncate()
            * if paddle.is_some() {
                Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)
            } else if wall.is_some() {
                // Walls use custom_size in the sprite, but transform.scale is 1.0
                // We need to figure out the wall size from its sprite custom_size.
                // Since we can't easily query Sprite here, use the wall dimensions directly.
                let diff = (target_pos - Vec2::ZERO).abs();
                if diff.y > diff.x {
                    // Left or right wall
                    Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT + WALL_THICKNESS * 2.0)
                } else {
                    // Top wall
                    Vec2::new(WINDOW_WIDTH + WALL_THICKNESS * 2.0, WALL_THICKNESS)
                }
            } else {
                continue;
            };

        if let Some(collision) = check_aabb_collision(ball_pos, ball_size, target_pos, target_size)
        {
            match collision {
                CollisionSide::Top | CollisionSide::Bottom => {
                    ball.velocity.y = -ball.velocity.y;
                }
                CollisionSide::Left | CollisionSide::Right => {
                    ball.velocity.x = -ball.velocity.x;
                }
            }

            // If hitting paddle, adjust angle based on where ball hit
            if paddle.is_some() {
                let hit_offset = (ball_pos.x - target_pos.x) / (PADDLE_WIDTH / 2.0);
                let angle = hit_offset * std::f32::consts::FRAC_PI_4; // max ±45° offset
                let speed = ball.velocity.length();
                ball.velocity = Vec2::new(
                    speed * angle.sin() + ball.velocity.x * 0.3,
                    ball.velocity.y.abs(), // Always bounce up
                )
                .normalize()
                    * speed;
            }

            // Push ball out of collision to avoid sticking
            match collision {
                CollisionSide::Top => {
                    ball_transform.translation.y =
                        target_pos.y + target_size.y / 2.0 + BALL_SIZE / 2.0 + 0.1;
                }
                CollisionSide::Bottom => {
                    ball_transform.translation.y =
                        target_pos.y - target_size.y / 2.0 - BALL_SIZE / 2.0 - 0.1;
                }
                CollisionSide::Left => {
                    ball_transform.translation.x =
                        target_pos.x - target_size.x / 2.0 - BALL_SIZE / 2.0 - 0.1;
                }
                CollisionSide::Right => {
                    ball_transform.translation.x =
                        target_pos.x + target_size.x / 2.0 + BALL_SIZE / 2.0 + 0.1;
                }
            }

            // Only handle one collision per frame
            break;
        }
    }
}

/// Ball vs bricks — destroy brick, reflect, and add score.
pub fn ball_collision_bricks(
    mut commands: Commands,
    mut ball_query: Query<(&mut Transform, &mut Ball)>,
    brick_query: Query<(Entity, &Transform), (With<Brick>, Without<Ball>)>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    let Ok((mut ball_transform, mut ball)) = ball_query.single_mut() else {
        return;
    };

    let ball_pos = ball_transform.translation.truncate();
    let ball_size = Vec2::splat(BALL_SIZE);
    let brick_size = Vec2::new(BRICK_WIDTH, BRICK_HEIGHT);

    for (brick_entity, brick_transform) in &brick_query {
        let brick_pos = brick_transform.translation.truncate();

        if let Some(collision) = check_aabb_collision(ball_pos, ball_size, brick_pos, brick_size) {
            commands.entity(brick_entity).despawn();
            scoreboard.score += POINTS_PER_BRICK;

            match collision {
                CollisionSide::Top | CollisionSide::Bottom => {
                    ball.velocity.y = -ball.velocity.y;
                }
                CollisionSide::Left | CollisionSide::Right => {
                    ball.velocity.x = -ball.velocity.x;
                }
            }

            // Push ball out
            match collision {
                CollisionSide::Top => {
                    ball_transform.translation.y =
                        brick_pos.y + brick_size.y / 2.0 + BALL_SIZE / 2.0 + 0.1;
                }
                CollisionSide::Bottom => {
                    ball_transform.translation.y =
                        brick_pos.y - brick_size.y / 2.0 - BALL_SIZE / 2.0 - 0.1;
                }
                CollisionSide::Left => {
                    ball_transform.translation.x =
                        brick_pos.x - brick_size.x / 2.0 - BALL_SIZE / 2.0 - 0.1;
                }
                CollisionSide::Right => {
                    ball_transform.translation.x =
                        brick_pos.x + brick_size.x / 2.0 + BALL_SIZE / 2.0 + 0.1;
                }
            }

            // Only handle one brick collision per frame
            break;
        }
    }
}

/// Detects when the ball falls below the screen (death zone).
pub fn ball_death_zone(
    mut ball_query: Query<(&mut Transform, &mut Ball)>,
    mut lives: ResMut<Lives>,
) {
    let Ok((mut ball_transform, mut ball)) = ball_query.single_mut() else {
        return;
    };

    let death_y = -WINDOW_HEIGHT / 2.0 - BALL_SIZE;

    if ball_transform.translation.y < death_y {
        lives.count = lives.count.saturating_sub(1);

        // Reset ball position
        ball_transform.translation.x = 0.0;
        ball_transform.translation.y = PADDLE_Y + PADDLE_HEIGHT / 2.0 + BALL_SIZE / 2.0 + 1.0;
        ball.velocity = Vec2::new(BALL_SPEED * 0.7, BALL_SPEED);
    }
}
