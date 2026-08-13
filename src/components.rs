use bevy::prelude::*;

// --- Game State ---

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
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

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct QuitButton;

// --- Pause Menu ---

/// Tracks which button is currently selected in the pause menu (for keyboard navigation).
#[derive(Resource, Default)]
pub struct PauseMenuState {
    pub selected: usize, // 0 = Resume, 1 = Quit
}

/// Number of items in the pause menu.
pub const PAUSE_MENU_ITEMS: usize = 2;

// Pause menu button colors
pub const BUTTON_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BUTTON_HOVERED: Color = Color::srgb(0.35, 0.35, 0.35);
pub const BUTTON_PRESSED: Color = Color::srgb(0.7, 0.6, 0.1);

// --- Actions ---

/// The paddle intent for the current simulation tick.
///
/// Both the keyboard input system and an external agent write to this resource;
/// the paddle movement system is the only reader.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaddleAction {
    Left,
    #[default]
    Stay,
    Right,
}

impl PaddleAction {
    /// Returns the horizontal direction (-1, 0 or 1) for this action.
    pub fn direction(self) -> f32 {
        match self {
            PaddleAction::Left => -1.0,
            PaddleAction::Stay => 0.0,
            PaddleAction::Right => 1.0,
        }
    }

    /// Returns all actions in a stable order (useful for agents).
    pub fn all() -> [PaddleAction; 3] {
        [PaddleAction::Left, PaddleAction::Stay, PaddleAction::Right]
    }
}

// --- Resources ---

#[derive(Resource, Default)]
pub struct Scoreboard {
    pub score: u32,
}

#[derive(Resource)]
pub struct Lives {
    pub count: u32,
}

impl Default for Lives {
    fn default() -> Self {
        Self {
            count: STARTING_LIVES,
        }
    }
}

/// Deterministic launch direction generator for the ball.
///
/// With `randomize` disabled the ball always starts along the same fixed
/// trajectory; when enabled the direction is jittered using a seeded PRNG so
/// episodes stay reproducible for a given seed.
#[derive(Resource, Debug, Clone, Copy)]
pub struct BallLaunch {
    pub randomize: bool,
    seed: u64,
    state: u64,
}

impl Default for BallLaunch {
    fn default() -> Self {
        Self::new(DEFAULT_SEED, false)
    }
}

impl BallLaunch {
    /// Creates a launcher with the given seed and randomization setting.
    pub fn new(seed: u64, randomize: bool) -> Self {
        Self {
            randomize,
            seed,
            state: seed.wrapping_add(BALL_LAUNCH_SEED_OFFSET),
        }
    }

    /// Restores the launcher to its initial (post-seed) state.
    pub fn reseed(&mut self) {
        self.state = self.seed.wrapping_add(BALL_LAUNCH_SEED_OFFSET);
    }

    /// Returns the next launch velocity for the ball.
    pub fn next_velocity(&mut self) -> Vec2 {
        let base = Vec2::new(BALL_SPEED * BALL_LAUNCH_X_RATIO, BALL_SPEED);
        if !self.randomize {
            return base;
        }

        let angle = base.y.atan2(base.x) + (self.next_unit() * 2.0 - 1.0) * BALL_LAUNCH_JITTER;
        Vec2::new(angle.cos(), angle.sin()) * base.length()
    }

    /// Returns the next pseudo-random value in `0..1` (xorshift64*).
    fn next_unit(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        let value = x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40;
        value as f32 / (1u64 << 24) as f32
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
pub const BALL_LAUNCH_X_RATIO: f32 = 0.7;
pub const BALL_LAUNCH_JITTER: f32 = 0.25; // radians
pub const BALL_LAUNCH_SEED_OFFSET: u64 = 0x9E37_79B9_7F4A_7C15;

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

// Simulation
/// Fixed simulation timestep — gameplay never depends on real frame time.
pub const SIM_DT: f32 = 1.0 / 60.0;
/// Number of simulation ticks a single agent action is repeated for.
pub const DEFAULT_ACTION_REPEAT: u32 = 4;
/// Default seed used for deterministic episodes.
pub const DEFAULT_SEED: u64 = 0;
/// Lives an episode starts with.
pub const STARTING_LIVES: u32 = 3;

// Rewards
pub const REWARD_PER_BRICK: f32 = 1.0;
pub const REWARD_LIFE_LOST: f32 = -1.0;
pub const REWARD_VICTORY: f32 = 10.0;
pub const REWARD_GAME_OVER: f32 = -5.0;

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

    // --- PaddleAction ---

    #[test]
    fn paddle_action_default_is_stay() {
        assert_eq!(PaddleAction::default(), PaddleAction::Stay);
    }

    #[test]
    fn paddle_action_directions() {
        assert_eq!(PaddleAction::Left.direction(), -1.0);
        assert_eq!(PaddleAction::Stay.direction(), 0.0);
        assert_eq!(PaddleAction::Right.direction(), 1.0);
    }

    #[test]
    fn paddle_action_all_lists_every_variant() {
        assert_eq!(
            PaddleAction::all(),
            [PaddleAction::Left, PaddleAction::Stay, PaddleAction::Right]
        );
    }

    // --- BallLaunch ---

    #[test]
    fn ball_launch_is_fixed_without_randomization() {
        let mut launch = BallLaunch::new(7, false);
        let first = launch.next_velocity();
        let second = launch.next_velocity();
        assert_eq!(first, second);
        assert_eq!(
            first,
            Vec2::new(BALL_SPEED * BALL_LAUNCH_X_RATIO, BALL_SPEED)
        );
    }

    #[test]
    fn ball_launch_randomizes_but_keeps_speed_and_upward_motion() {
        let mut launch = BallLaunch::new(42, true);
        let mut previous = None;
        let mut any_different = false;

        for _ in 0..10 {
            let velocity = launch.next_velocity();
            let speed = Vec2::new(BALL_SPEED * BALL_LAUNCH_X_RATIO, BALL_SPEED).length();
            assert!((velocity.length() - speed).abs() < 0.01, "Speed preserved");
            assert!(velocity.y > 0.0, "Ball should always launch upward");
            if let Some(prev) = previous
                && prev != velocity
            {
                any_different = true;
            }
            previous = Some(velocity);
        }

        assert!(any_different, "Randomized launches should vary");
    }

    #[test]
    fn ball_launch_is_deterministic_for_a_seed() {
        let mut a = BallLaunch::new(123, true);
        let mut b = BallLaunch::new(123, true);
        for _ in 0..5 {
            assert_eq!(a.next_velocity(), b.next_velocity());
        }

        // Reseeding restarts the exact same sequence
        a.reseed();
        let mut fresh = BallLaunch::new(123, true);
        for _ in 0..5 {
            assert_eq!(a.next_velocity(), fresh.next_velocity());
        }
    }

    // --- Simulation constants ---

    #[test]
    fn simulation_constants_valid() {
        assert!(SIM_DT > 0.0);
        assert!(DEFAULT_ACTION_REPEAT >= 1);
        assert!(STARTING_LIVES > 0);
    }
}
