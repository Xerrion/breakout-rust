use bevy::prelude::*;

// --- Game State ---

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
    Victory,
}

// --- Components ---

#[derive(Component)]
pub struct Paddle;

#[derive(Component)]
pub struct Ball {
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct Collider;

#[derive(Component)]
pub struct Wall;

// --- UI Markers ---

#[derive(Component)]
pub struct ScoreboardUi;

#[derive(Component)]
pub struct LivesUi;

#[derive(Component)]
pub struct OverlayUi;

// --- Resources ---

#[derive(Resource)]
pub struct Scoreboard {
    pub score: u32,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self { score: 0 }
    }
}

#[derive(Resource)]
pub struct Lives {
    pub count: u32,
}

impl Default for Lives {
    fn default() -> Self {
        Self { count: 3 }
    }
}

// --- Shared Constants ---

// Window
pub const WINDOW_WIDTH: f32 = 900.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

// Paddle
pub const PADDLE_WIDTH: f32 = 120.0;
pub const PADDLE_HEIGHT: f32 = 20.0;
pub const PADDLE_Y: f32 = -WINDOW_HEIGHT / 2.0 + 40.0;
pub const PADDLE_SPEED: f32 = 500.0;
pub const PADDLE_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

// Ball
pub const BALL_SIZE: f32 = 16.0;
pub const BALL_SPEED: f32 = 350.0;
pub const BALL_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

// Bricks
pub const BRICK_WIDTH: f32 = 80.0;
pub const BRICK_HEIGHT: f32 = 30.0;
pub const BRICK_GAP: f32 = 4.0;
pub const BRICK_COLS: usize = 10;
pub const BRICK_ROWS: usize = 5;
pub const BRICK_COLORS: [Color; 5] = [
    Color::srgb(0.9, 0.2, 0.2), // Red
    Color::srgb(0.9, 0.6, 0.1), // Orange
    Color::srgb(0.9, 0.9, 0.2), // Yellow
    Color::srgb(0.2, 0.8, 0.2), // Green
    Color::srgb(0.3, 0.5, 0.9), // Blue
];
pub const POINTS_PER_BRICK: u32 = 10;

// Walls
pub const WALL_THICKNESS: f32 = 10.0;
pub const WALL_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

// --- Collision Helper ---

#[derive(Debug, PartialEq)]
pub enum CollisionSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// AABB collision check between two rectangles.
/// Returns the side of `target` that was hit, if any.
pub fn check_aabb_collision(
    ball_pos: Vec2,
    ball_size: Vec2,
    target_pos: Vec2,
    target_size: Vec2,
) -> Option<CollisionSide> {
    let ball_half = ball_size / 2.0;
    let target_half = target_size / 2.0;

    let diff = ball_pos - target_pos;
    let overlap_x = ball_half.x + target_half.x - diff.x.abs();
    let overlap_y = ball_half.y + target_half.y - diff.y.abs();

    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return None;
    }

    if overlap_x < overlap_y {
        if diff.x > 0.0 {
            Some(CollisionSide::Right)
        } else {
            Some(CollisionSide::Left)
        }
    } else if diff.y > 0.0 {
        Some(CollisionSide::Top)
    } else {
        Some(CollisionSide::Bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- check_aabb_collision tests ---

    #[test]
    fn no_collision_when_far_apart() {
        let result = check_aabb_collision(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(10.0, 10.0),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn no_collision_when_touching_edge() {
        // Exactly touching: overlap = 0, which is <= 0, so no collision
        let result = check_aabb_collision(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn collision_from_top() {
        // Ball is above the target, slightly overlapping
        let result = check_aabb_collision(
            Vec2::new(0.0, 14.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 20.0),
        );
        assert_eq!(result, Some(CollisionSide::Top));
    }

    #[test]
    fn collision_from_bottom() {
        // Ball is below the target, slightly overlapping
        let result = check_aabb_collision(
            Vec2::new(0.0, -14.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 20.0),
        );
        assert_eq!(result, Some(CollisionSide::Bottom));
    }

    #[test]
    fn collision_from_left() {
        // Ball is to the left, slightly overlapping
        let result = check_aabb_collision(
            Vec2::new(-14.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 20.0),
        );
        assert_eq!(result, Some(CollisionSide::Left));
    }

    #[test]
    fn collision_from_right() {
        // Ball is to the right, slightly overlapping
        let result = check_aabb_collision(
            Vec2::new(14.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 20.0),
        );
        assert_eq!(result, Some(CollisionSide::Right));
    }

    #[test]
    fn collision_prefers_x_axis_when_x_overlap_smaller() {
        // Ball overlaps target with smaller x-overlap than y-overlap
        // Ball at x=9, target at x=0, both 10 wide: overlap_x = 5+5-9 = 1
        // Ball at y=0, target at y=0, both 10 tall: overlap_y = 5+5-0 = 10
        // overlap_x < overlap_y → returns Right (diff.x > 0)
        let result = check_aabb_collision(
            Vec2::new(9.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
        );
        assert_eq!(result, Some(CollisionSide::Right));
    }

    #[test]
    fn collision_prefers_y_axis_when_y_overlap_smaller() {
        // Ball overlaps target with smaller y-overlap than x-overlap
        // Ball at x=0, target at x=0, both 10 wide: overlap_x = 10
        // Ball at y=9, target at y=0, both 10 tall: overlap_y = 5+5-9 = 1
        // overlap_x > overlap_y → returns Top (diff.y > 0)
        let result = check_aabb_collision(
            Vec2::new(0.0, 9.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
        );
        assert_eq!(result, Some(CollisionSide::Top));
    }

    // --- Constant sanity checks ---

    #[test]
    fn window_dimensions_positive() {
        assert!(WINDOW_WIDTH > 0.0);
        assert!(WINDOW_HEIGHT > 0.0);
    }

    #[test]
    fn entity_dimensions_positive() {
        assert!(PADDLE_WIDTH > 0.0);
        assert!(PADDLE_HEIGHT > 0.0);
        assert!(BALL_SIZE > 0.0);
        assert!(BRICK_WIDTH > 0.0);
        assert!(BRICK_HEIGHT > 0.0);
        assert!(WALL_THICKNESS > 0.0);
        assert!(PADDLE_SPEED > 0.0);
        assert!(BALL_SPEED > 0.0);
    }

    #[test]
    fn brick_grid_fits_in_window() {
        let grid_width = BRICK_COLS as f32 * (BRICK_WIDTH + BRICK_GAP) - BRICK_GAP;
        assert!(grid_width < WINDOW_WIDTH, "Brick grid wider than window");
    }

    #[test]
    fn default_resources_valid() {
        let scoreboard = Scoreboard::default();
        assert_eq!(scoreboard.score, 0);

        let lives = Lives::default();
        assert!(lives.count > 0);
    }
}
