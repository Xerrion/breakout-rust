use bevy::app::TaskPoolPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::state::state::StateTransition;

use crate::components::*;
use crate::gameplay::GameplayPlugin;

// --- Reward Constants ---

/// Reward granted per destroyed brick.
pub const REWARD_PER_BRICK: f32 = 1.0;
/// Reward granted when a life is lost.
pub const REWARD_LIFE_LOST: f32 = -1.0;
/// Reward granted when the episode ends in victory.
pub const REWARD_VICTORY: f32 = 10.0;
/// Reward granted when the episode ends in game over.
pub const REWARD_GAME_OVER: f32 = -10.0;

// --- Environment Types ---

/// Configuration of a headless training environment.
#[derive(Debug, Clone, Copy)]
pub struct EnvConfig {
    /// Number of simulation ticks a single action is applied for.
    pub action_repeat: u32,
    /// Maximum random angle offset (radians) applied to the ball launch on reset.
    pub ball_angle_jitter: f32,
    /// Seed of the deterministic simulation RNG.
    pub seed: u64,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            action_repeat: 1,
            ball_angle_jitter: 0.0,
            seed: DEFAULT_SEED,
        }
    }
}

/// Normalized snapshot of the simulation state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Observation {
    /// Paddle x position, normalized to -1..1.
    pub paddle_x: f32,
    /// Ball x position, normalized to -1..1.
    pub ball_x: f32,
    /// Ball y position, normalized to -1..1.
    pub ball_y: f32,
    /// Ball x velocity, normalized to -1..1.
    pub ball_velocity_x: f32,
    /// Ball y velocity, normalized to -1..1.
    pub ball_velocity_y: f32,
    /// Remaining bricks, normalized to 0..1.
    pub bricks_remaining: f32,
    /// Remaining lives, normalized to 0..1.
    pub lives: f32,
}

impl Observation {
    /// Returns the observation as a flat array, ready for a neural network.
    pub fn to_array(self) -> [f32; 7] {
        [
            self.paddle_x,
            self.ball_x,
            self.ball_y,
            self.ball_velocity_x,
            self.ball_velocity_y,
            self.bricks_remaining,
            self.lives,
        ]
    }
}

/// Result of a single environment step.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StepResult {
    pub observation: Observation,
    pub reward: f32,
    pub terminated: bool,
}

/// Headless Breakout environment with a `reset` / `step` / `observation` interface.
///
/// Runs the exact same gameplay systems as the rendered game, but without
/// window, rendering, UI, audio or frame limiter, and driven by an explicit
/// fixed timestep.
pub struct BreakoutEnv {
    app: App,
    config: EnvConfig,
}

impl Default for BreakoutEnv {
    fn default() -> Self {
        Self::new(EnvConfig::default())
    }
}

impl BreakoutEnv {
    /// Creates a headless environment and resets it to a known start state.
    pub fn new(config: EnvConfig) -> Self {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), StatesPlugin, GameplayPlugin))
            .insert_resource(SimRng::new(config.seed))
            .insert_resource(BallLaunch {
                angle_jitter: config.ball_angle_jitter,
            });

        let mut env = Self { app, config };
        env.reset();
        env
    }

    /// Returns the environment configuration.
    pub fn config(&self) -> EnvConfig {
        self.config
    }

    /// Starts a new episode and returns the initial observation.
    pub fn reset(&mut self) -> Observation {
        // Going Menu -> Playing runs the same complete reset as the real game.
        self.set_state(GameState::Menu);
        self.set_state(GameState::Playing);
        self.observation()
    }

    /// Applies `action` for `action_repeat` simulation ticks.
    pub fn step(&mut self, action: PaddleAction) -> StepResult {
        self.app.world_mut().insert_resource(action);

        let score_before = self.score();
        let lives_before = self.lives();

        let mut terminated = self.terminated();
        for _ in 0..self.config.action_repeat.max(1) {
            if terminated {
                break;
            }
            self.tick();
            terminated = self.terminated();
        }

        let bricks = (self.score() - score_before) / POINTS_PER_BRICK;
        let lost_lives = lives_before.saturating_sub(self.lives());

        let mut reward = bricks as f32 * REWARD_PER_BRICK + lost_lives as f32 * REWARD_LIFE_LOST;
        match self.state() {
            GameState::Victory => reward += REWARD_VICTORY,
            GameState::GameOver => reward += REWARD_GAME_OVER,
            _ => {}
        }

        StepResult {
            observation: self.observation(),
            reward,
            terminated,
        }
    }

    /// Returns the current normalized observation.
    pub fn observation(&self) -> Observation {
        let world = self.app.world();
        let mut observation = Observation {
            lives: self.lives() as f32 / INITIAL_LIVES as f32,
            ..default()
        };

        let mut paddles = world.try_query_filtered::<&Transform, With<Paddle>>();
        if let Some(query) = paddles.as_mut()
            && let Some(transform) = query.iter(world).next()
        {
            observation.paddle_x = normalize(transform.translation.x, WINDOW_WIDTH / 2.0);
        }

        let mut balls = world.try_query::<(&Transform, &Ball)>();
        if let Some(query) = balls.as_mut()
            && let Some((transform, ball)) = query.iter(world).next()
        {
            observation.ball_x = normalize(transform.translation.x, WINDOW_WIDTH / 2.0);
            observation.ball_y = normalize(transform.translation.y, WINDOW_HEIGHT / 2.0);
            observation.ball_velocity_x = normalize(ball.velocity.x, BALL_MAX_SPEED);
            observation.ball_velocity_y = normalize(ball.velocity.y, BALL_MAX_SPEED);
        }

        observation.bricks_remaining =
            self.bricks_remaining() as f32 / (BRICK_ROWS * BRICK_COLS) as f32;

        observation
    }

    /// Returns whether the current episode has ended.
    pub fn terminated(&self) -> bool {
        matches!(self.state(), GameState::GameOver | GameState::Victory)
    }

    /// Returns the current score.
    pub fn score(&self) -> u32 {
        self.app.world().resource::<Scoreboard>().score
    }

    /// Returns the remaining lives.
    pub fn lives(&self) -> u32 {
        self.app.world().resource::<Lives>().count
    }

    /// Returns the number of bricks left on the board.
    pub fn bricks_remaining(&self) -> usize {
        let world = self.app.world();
        let Some(mut query) = world.try_query_filtered::<Entity, With<Brick>>() else {
            return 0;
        };
        query.iter(world).count()
    }

    /// Returns the current game state.
    pub fn state(&self) -> GameState {
        *self.app.world().resource::<State<GameState>>().get()
    }

    /// Advances the simulation by exactly one fixed timestep.
    fn tick(&mut self) {
        let world = self.app.world_mut();
        world.run_schedule(FixedUpdate);
        world.run_schedule(StateTransition);
    }

    /// Transitions to `state` and applies the transition immediately.
    fn set_state(&mut self, state: GameState) {
        let world = self.app.world_mut();
        world.resource_mut::<NextState<GameState>>().set(state);
        world.run_schedule(StateTransition);
    }
}

/// Normalizes `value` into -1..1 (or 0..1 for non-negative values) using `scale`.
fn normalize(value: f32, scale: f32) -> f32 {
    (value / scale).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> BreakoutEnv {
        BreakoutEnv::default()
    }

    // --- reset ---

    #[test]
    fn reset_returns_known_start_state() {
        let mut env = env();
        let observation = env.reset();

        assert_eq!(env.state(), GameState::Playing);
        assert_eq!(env.score(), 0);
        assert_eq!(env.lives(), INITIAL_LIVES);
        assert_eq!(env.bricks_remaining(), BRICK_ROWS * BRICK_COLS);
        assert!(observation.paddle_x.abs() < f32::EPSILON);
        assert!(observation.bricks_remaining == 1.0);
        assert!(observation.lives == 1.0);
    }

    #[test]
    fn reset_after_playing_restores_start_state() {
        let mut env = env();
        for _ in 0..300 {
            env.step(PaddleAction::Right);
        }

        let before_reset = env.observation();
        let observation = env.reset();

        assert_eq!(env.score(), 0);
        assert_eq!(env.lives(), INITIAL_LIVES);
        assert_eq!(env.bricks_remaining(), BRICK_ROWS * BRICK_COLS);
        assert!(!env.terminated());
        assert!(observation.paddle_x.abs() < f32::EPSILON);
        assert_ne!(
            before_reset, observation,
            "Reset should return to the start state"
        );
    }

    #[test]
    fn reset_is_deterministic() {
        let mut first = env();
        let mut second = env();

        for _ in 0..200 {
            first.step(PaddleAction::Left);
            second.step(PaddleAction::Left);
        }

        assert_eq!(first.observation(), second.observation());
        assert_eq!(first.score(), second.score());
    }

    // --- observation ---

    #[test]
    fn observation_values_are_normalized() {
        let mut env = env();
        env.reset();

        for _ in 0..600 {
            let result = env.step(PaddleAction::Right);
            for value in result.observation.to_array() {
                assert!(
                    (-1.0..=1.0).contains(&value),
                    "Observation value out of range: {value}"
                );
            }
            if result.terminated {
                break;
            }
        }
    }

    #[test]
    fn observation_tracks_ball_and_paddle() {
        let mut env = env();
        env.reset();

        let start = env.observation();
        env.step(PaddleAction::Right);
        let moved = env.observation();

        assert!(moved.paddle_x > start.paddle_x, "Paddle should move right");
        assert!(moved.ball_y > start.ball_y, "Ball should move upward");
        assert!(moved.ball_velocity_y > 0.0);
    }

    // --- actions ---

    #[test]
    fn actions_move_paddle_in_both_directions() {
        let mut env = env();
        env.reset();

        env.step(PaddleAction::Left);
        let left = env.observation().paddle_x;
        assert!(left < 0.0, "Left action should move paddle left");

        env.step(PaddleAction::Stay);
        assert!(
            (env.observation().paddle_x - left).abs() < f32::EPSILON,
            "Stay action should not move the paddle"
        );

        env.step(PaddleAction::Right);
        assert!(
            env.observation().paddle_x > left,
            "Right action should move paddle right"
        );
    }

    #[test]
    fn action_repeat_applies_action_multiple_ticks() {
        let mut single = BreakoutEnv::new(EnvConfig::default());
        let mut repeated = BreakoutEnv::new(EnvConfig {
            action_repeat: 4,
            ..EnvConfig::default()
        });

        single.step(PaddleAction::Right);
        repeated.step(PaddleAction::Right);

        let single_x = single.observation().paddle_x;
        let repeated_x = repeated.observation().paddle_x;
        assert!(
            repeated_x > single_x * 3.0,
            "Action repeat should apply the action for several ticks"
        );
    }

    // --- rewards ---

    #[test]
    fn reward_is_zero_without_events() {
        let mut env = env();
        let result = env.step(PaddleAction::Stay);
        assert_eq!(result.reward, 0.0);
        assert!(!result.terminated);
    }

    #[test]
    fn reward_is_positive_when_bricks_are_destroyed() {
        let mut env = env();
        let mut total = 0.0;

        for _ in 0..2000 {
            let result = env.step(PaddleAction::Stay);
            total += result.reward;
            if env.score() > 0 {
                break;
            }
        }

        assert!(env.score() > 0, "Ball should eventually hit a brick");
        assert!(total > 0.0, "Destroying bricks should yield reward");
    }

    #[test]
    fn reward_is_negative_when_life_is_lost() {
        let mut env = env();
        let lives_before = env.lives();
        let mut reward_on_death = 0.0;

        for _ in 0..5000 {
            let result = env.step(PaddleAction::Left);
            if env.lives() < lives_before {
                reward_on_death = result.reward;
                break;
            }
        }

        assert!(env.lives() < lives_before, "Ball should be missed");
        assert!(
            reward_on_death <= REWARD_LIFE_LOST,
            "Losing a life should be penalized, got {reward_on_death}"
        );
    }

    // --- terminal states ---

    #[test]
    fn episode_terminates_on_game_over() {
        let mut env = env();

        let mut terminated = false;
        for _ in 0..20000 {
            if env.step(PaddleAction::Left).terminated {
                terminated = true;
                break;
            }
        }

        assert!(terminated, "Episode should terminate");
        assert_eq!(env.state(), GameState::GameOver);
        assert_eq!(env.lives(), 0);
    }

    #[test]
    fn stepping_after_termination_does_not_advance() {
        let mut env = env();

        for _ in 0..20000 {
            if env.step(PaddleAction::Left).terminated {
                break;
            }
        }

        let observation = env.observation();
        let result = env.step(PaddleAction::Right);

        assert!(result.terminated);
        assert_eq!(result.observation, observation);
        assert_eq!(result.reward, REWARD_GAME_OVER);
    }

    // --- ball launch randomization ---

    #[test]
    fn ball_angle_jitter_varies_launch_direction() {
        let mut env = BreakoutEnv::new(EnvConfig {
            ball_angle_jitter: 0.3,
            ..EnvConfig::default()
        });

        let first = env.observation();
        env.reset();
        let second = env.observation();

        assert_ne!(
            first.ball_velocity_x, second.ball_velocity_x,
            "Jitter should vary the launch direction between episodes"
        );
        assert!(first.ball_velocity_y > 0.0 && second.ball_velocity_y > 0.0);
    }
}
