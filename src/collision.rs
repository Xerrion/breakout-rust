use bevy::prelude::*;

use crate::components::*;

/// Ball vs walls and paddle — reflect velocity on collision.
#[allow(clippy::type_complexity)]
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
                if diff.x > diff.y {
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
#[allow(clippy::type_complexity)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Scoreboard>();
        app.init_resource::<Lives>();
        app
    }

    // --- Wall size heuristic regression test ---

    #[test]
    fn wall_size_heuristic_top_wall() {
        // Top wall at (0, 305): diff.x=0 < diff.y=305
        // Should get top-wall dimensions (wide and thin)
        let top_wall_pos = Vec2::new(0.0, WINDOW_HEIGHT / 2.0 + WALL_THICKNESS / 2.0);
        let diff = top_wall_pos.abs();
        let size = if diff.x > diff.y {
            Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT + WALL_THICKNESS * 2.0)
        } else {
            Vec2::new(WINDOW_WIDTH + WALL_THICKNESS * 2.0, WALL_THICKNESS)
        };
        assert!(
            size.x > size.y,
            "Top wall should be wider than tall, got {size:?}"
        );
    }

    #[test]
    fn wall_size_heuristic_side_walls() {
        // Left wall at (-455, 0): diff.x=455 > diff.y=0
        // Should get side-wall dimensions (thin and tall)
        let left_wall_pos = Vec2::new(-WINDOW_WIDTH / 2.0 - WALL_THICKNESS / 2.0, 0.0);
        let diff = left_wall_pos.abs();
        let size = if diff.x > diff.y {
            Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT + WALL_THICKNESS * 2.0)
        } else {
            Vec2::new(WINDOW_WIDTH + WALL_THICKNESS * 2.0, WALL_THICKNESS)
        };
        assert!(
            size.y > size.x,
            "Side wall should be taller than wide, got {size:?}"
        );
    }

    // --- ball_collision_walls_and_paddle ---

    #[test]
    fn ball_reflects_off_top_wall() {
        let mut app = test_app();
        app.add_systems(Update, ball_collision_walls_and_paddle);

        let top_wall_y = WINDOW_HEIGHT / 2.0 + WALL_THICKNESS / 2.0;

        // Spawn ball just below top wall, moving upward
        app.world_mut().spawn((
            Transform::from_xyz(
                0.0,
                top_wall_y - WALL_THICKNESS / 2.0 - BALL_SIZE / 2.0 + 2.0,
                1.0,
            ),
            Ball {
                velocity: Vec2::new(0.0, BALL_SPEED),
            },
        ));

        // Spawn top wall with Collider + Wall
        app.world_mut()
            .spawn((Transform::from_xyz(0.0, top_wall_y, 0.0), Wall, Collider));

        app.update();

        let mut q = app.world_mut().query::<&Ball>();
        let ball_vel = q.iter(app.world()).next().unwrap().velocity;
        assert!(
            ball_vel.y < 0.0,
            "Ball should bounce down after hitting top wall, got y={}",
            ball_vel.y
        );
    }

    #[test]
    fn ball_reflects_off_side_wall() {
        let mut app = test_app();
        app.add_systems(Update, ball_collision_walls_and_paddle);

        let right_wall_x = WINDOW_WIDTH / 2.0 + WALL_THICKNESS / 2.0;

        // Spawn ball near right wall, moving right
        app.world_mut().spawn((
            Transform::from_xyz(
                right_wall_x - WALL_THICKNESS / 2.0 - BALL_SIZE / 2.0 + 2.0,
                0.0,
                1.0,
            ),
            Ball {
                velocity: Vec2::new(BALL_SPEED, 0.0),
            },
        ));

        // Spawn right wall
        app.world_mut()
            .spawn((Transform::from_xyz(right_wall_x, 0.0, 0.0), Wall, Collider));

        app.update();

        let mut q = app.world_mut().query::<&Ball>();
        let ball_vel = q.iter(app.world()).next().unwrap().velocity;
        assert!(
            ball_vel.x < 0.0,
            "Ball should bounce left after hitting right wall, got x={}",
            ball_vel.x
        );
    }

    // --- ball_collision_bricks ---

    #[test]
    fn ball_destroys_brick_and_scores() {
        let mut app = test_app();
        app.add_systems(Update, ball_collision_bricks);

        // Spawn ball overlapping a brick
        let brick_y = 100.0;
        app.world_mut().spawn((
            Transform::from_xyz(
                0.0,
                brick_y - BRICK_HEIGHT / 2.0 - BALL_SIZE / 2.0 + 2.0,
                1.0,
            ),
            Ball {
                velocity: Vec2::new(0.0, BALL_SPEED),
            },
        ));

        // Spawn a brick
        app.world_mut()
            .spawn((Transform::from_xyz(0.0, brick_y, 0.0), Brick, Collider));

        app.update();

        let scoreboard = app.world().resource::<Scoreboard>();
        assert_eq!(scoreboard.score, POINTS_PER_BRICK);

        let mut q = app.world_mut().query::<&Brick>();
        let brick_count = q.iter(app.world()).count();
        assert_eq!(brick_count, 0, "Brick should be despawned after hit");
    }

    #[test]
    fn ball_reflects_on_brick_hit() {
        let mut app = test_app();
        app.add_systems(Update, ball_collision_bricks);

        // Ball moving upward, overlapping a brick from below
        let brick_y = 100.0;
        app.world_mut().spawn((
            Transform::from_xyz(
                0.0,
                brick_y - BRICK_HEIGHT / 2.0 - BALL_SIZE / 2.0 + 2.0,
                1.0,
            ),
            Ball {
                velocity: Vec2::new(100.0, BALL_SPEED),
            },
        ));

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, brick_y, 0.0), Brick, Collider));

        app.update();

        let mut q = app.world_mut().query::<&Ball>();
        let ball_vel = q.iter(app.world()).next().unwrap().velocity;
        assert!(
            ball_vel.y < 0.0,
            "Ball y-velocity should flip after hitting brick from below"
        );
    }

    // --- ball_death_zone ---

    #[test]
    fn ball_below_death_zone_loses_life() {
        let mut app = test_app();
        app.add_systems(Update, ball_death_zone);

        let death_y = -WINDOW_HEIGHT / 2.0 - BALL_SIZE;
        app.world_mut().spawn((
            Transform::from_xyz(50.0, death_y - 10.0, 1.0),
            Ball {
                velocity: Vec2::new(100.0, -BALL_SPEED),
            },
        ));

        app.update();

        let lives = app.world().resource::<Lives>();
        assert_eq!(lives.count, 2, "Should lose one life (3 -> 2)");

        // Ball should reset to center
        let mut q = app.world_mut().query::<(&Transform, &Ball)>();
        let ball_transform = q.iter(app.world()).next().unwrap().0;
        assert!(
            (ball_transform.translation.x).abs() < 0.01,
            "Ball x should reset to 0"
        );
    }

    #[test]
    fn ball_death_zone_saturates_at_zero() {
        let mut app = test_app();
        app.add_systems(Update, ball_death_zone);
        app.world_mut().resource_mut::<Lives>().count = 0;

        let death_y = -WINDOW_HEIGHT / 2.0 - BALL_SIZE;
        app.world_mut().spawn((
            Transform::from_xyz(0.0, death_y - 10.0, 1.0),
            Ball {
                velocity: Vec2::new(0.0, -BALL_SPEED),
            },
        ));

        app.update();

        let lives = app.world().resource::<Lives>();
        assert_eq!(lives.count, 0, "Lives should stay at 0 via saturating_sub");
    }

    #[test]
    fn ball_above_death_zone_keeps_lives() {
        let mut app = test_app();
        app.add_systems(Update, ball_death_zone);

        // Ball well above death zone
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            Ball {
                velocity: Vec2::new(100.0, BALL_SPEED),
            },
        ));

        app.update();

        let lives = app.world().resource::<Lives>();
        assert_eq!(lives.count, 3, "Lives should remain unchanged");
    }
}
