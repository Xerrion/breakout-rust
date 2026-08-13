use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

use crate::components::*;
use crate::gameplay::{GameplayPlugin, Simulation};
use crate::setup;

/// Configuration for a headless training environment.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvConfig {
    /// Number of simulation ticks a single action is applied for.
    pub action_repeat: u32,
    /// Seed used for the ball launch direction.
    pub seed: u64,
    /// Whether the ball launch direction is jittered on reset.
    pub randomize_ball_direction: bool,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            action_repeat: DEFAULT_ACTION_REPEAT,
            seed: DEFAULT_SEED,
            randomize_ball_direction: false,
        }
    }
}

/// Normalized view of the game state for an agent.
///
/// Positions are normalized to roughly `-1..1`, velocities to `-1..1` and the
/// remaining brick fraction to `0..1`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Observation {
    pub paddle_x: f32,
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_velocity_x: f32,
    pub ball_velocity_y: f32,
    pub bricks_remaining: f32,
}

impl Observation {
    /// Number of values in an observation.
    pub const LEN: usize = 6;

    /// Returns the observation as a flat array, ready for a neural network.
    pub fn to_array(self) -> [f32; Self::LEN] {
        [
            self.paddle_x,
            self.ball_x,
            self.ball_y,
            self.ball_velocity_x,
            self.ball_velocity_y,
            self.bricks_remaining,
        ]
    }
}

/// Result of a single environment step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepResult {
    pub observation: Observation,
    pub reward: f32,
    pub terminated: bool,
}

/// Headless environment wrapping the gameplay simulation.
///
/// The environment owns a Bevy app built without window, rendering, UI, audio
/// or frame limiter, and exposes the classic `reset` / `step` / `observation`
/// interface used by reinforcement-learning agents.
pub struct BreakoutEnv {
    app: App,
    config: EnvConfig,
    previous_score: u32,
    previous_lives: u32,
}

impl Default for BreakoutEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl BreakoutEnv {
    /// Creates an environment with the default configuration.
    pub fn new() -> Self {
        Self::with_config(EnvConfig::default())
    }

    /// Creates an environment with the given configuration.
    pub fn with_config(config: EnvConfig) -> Self {
        let mut env = Self {
            app: headless_app(),
            config,
            previous_score: 0,
            previous_lives: STARTING_LIVES,
        };
        env.app.update();
        env.app.world_mut().insert_resource(BallLaunch::new(
            config.seed,
            config.randomize_ball_direction,
        ));
        env.reset();
        env
    }

    /// Returns the environment configuration.
    pub fn config(&self) -> EnvConfig {
        self.config
    }

    /// Resets the episode to a known start state and returns the first observation.
    ///
    /// The reset is complete and deterministic: score, lives, paddle, ball,
    /// bricks and game state all return to their start values. When ball
    /// randomization is enabled the launch direction follows the seeded random
    /// stream, so episodes vary but remain reproducible for a given seed.
    pub fn reset(&mut self) -> Observation {
        // Clear and rebuild the board
        let world = self.app.world_mut();
        let _ = world.run_system_cached(setup::despawn_game_entities);
        let _ = world.run_system_cached(setup::spawn_game);

        // Reset gameplay resources
        world.insert_resource(Scoreboard::default());
        world.insert_resource(Lives::default());
        world.insert_resource(PaddleAction::default());
        if !self.config.randomize_ball_direction {
            world.resource_mut::<BallLaunch>().reseed();
        }

        // Force a Menu -> Playing transition so OnEnter(Playing) always runs
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Menu);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        self.app.update();

        self.previous_score = 0;
        self.previous_lives = STARTING_LIVES;

        self.observation()
    }

    /// Reseeds the environment and resets the episode.
    pub fn reset_with_seed(&mut self, seed: u64) -> Observation {
        self.config.seed = seed;
        self.app
            .world_mut()
            .insert_resource(BallLaunch::new(seed, self.config.randomize_ball_direction));
        self.reset()
    }

    /// Applies an action for `action_repeat` simulation ticks.
    pub fn step(&mut self, action: PaddleAction) -> StepResult {
        self.app.world_mut().insert_resource(action);

        let mut reward = 0.0;
        let mut terminated = self.terminated();

        for _ in 0..self.config.action_repeat.max(1) {
            if terminated {
                break;
            }
            self.tick();
            reward += self.collect_reward();
            terminated = self.terminated();
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

        let paddle_x = world
            .iter_entities()
            .filter(|entity| entity.contains::<Paddle>())
            .filter_map(|entity| entity.get::<Transform>())
            .map(|transform| transform.translation.x)
            .next()
            .unwrap_or(0.0);

        let ball = world
            .iter_entities()
            .filter(|entity| entity.contains::<Ball>())
            .filter_map(|entity| Some((entity.get::<Transform>()?, entity.get::<Ball>()?)))
            .next();

        let bricks = world
            .iter_entities()
            .filter(|entity| entity.contains::<Brick>())
            .count();

        let (ball_pos, ball_velocity) = match ball {
            Some((transform, ball)) => (transform.translation.truncate(), ball.velocity),
            None => (Vec2::ZERO, Vec2::ZERO),
        };

        let half_width = WINDOW_WIDTH / 2.0;
        let half_height = WINDOW_HEIGHT / 2.0;
        let total_bricks = (BRICK_ROWS * BRICK_COLS) as f32;

        Observation {
            paddle_x: (paddle_x / half_width).clamp(-1.0, 1.0),
            ball_x: (ball_pos.x / half_width).clamp(-1.0, 1.0),
            ball_y: (ball_pos.y / half_height).clamp(-1.0, 1.0),
            ball_velocity_x: (ball_velocity.x / BALL_SPEED).clamp(-1.0, 1.0),
            ball_velocity_y: (ball_velocity.y / BALL_SPEED).clamp(-1.0, 1.0),
            bricks_remaining: bricks as f32 / total_bricks,
        }
    }

    /// Returns the current score.
    pub fn score(&self) -> u32 {
        self.app.world().resource::<Scoreboard>().score
    }

    /// Returns the remaining lives.
    pub fn lives(&self) -> u32 {
        self.app.world().resource::<Lives>().count
    }

    /// Returns the current game state.
    pub fn state(&self) -> GameState {
        *self.app.world().resource::<State<GameState>>().get()
    }

    /// Returns whether the episode has reached a terminal state.
    pub fn terminated(&self) -> bool {
        matches!(self.state(), GameState::GameOver | GameState::Victory)
    }

    /// Advances the simulation by exactly one fixed tick.
    fn tick(&mut self) {
        self.app.world_mut().run_schedule(Simulation);
        // Applies pending state transitions triggered by the simulation
        self.app.update();
    }

    /// Computes the reward from the change in game state since the last tick.
    fn collect_reward(&mut self) -> f32 {
        let score = self.score();
        let lives = self.lives();

        let bricks_destroyed = score.saturating_sub(self.previous_score) / POINTS_PER_BRICK;
        let lives_lost = self.previous_lives.saturating_sub(lives);

        let mut reward =
            bricks_destroyed as f32 * REWARD_PER_BRICK + lives_lost as f32 * REWARD_LIFE_LOST;

        match self.state() {
            GameState::Victory => reward += REWARD_VICTORY,
            GameState::GameOver => reward += REWARD_GAME_OVER,
            _ => {}
        }

        self.previous_score = score;
        self.previous_lives = lives;

        reward
    }
}

/// Builds a headless app: gameplay simulation only, with no window, rendering,
/// UI, audio or frame limiter.
pub fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::ZERO)),
        GameplayPlugin,
    ));
    app
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> BreakoutEnv {
        BreakoutEnv::with_config(EnvConfig {
            action_repeat: 1,
            ..default()
        })
    }

    // --- reset ---

    #[test]
    fn reset_starts_in_playing_state() {
        let env = env();
        assert_eq!(env.state(), GameState::Playing);
        assert!(!env.terminated());
    }

    #[test]
    fn reset_restores_score_lives_and_bricks() {
        let mut env = env();

        // Corrupt the state
        for _ in 0..200 {
            env.step(PaddleAction::Left);
        }
        env.app.world_mut().resource_mut::<Scoreboard>().score = 999;
        env.app.world_mut().resource_mut::<Lives>().count = 1;

        let observation = env.reset();

        assert_eq!(env.score(), 0);
        assert_eq!(env.lives(), STARTING_LIVES);
        assert_eq!(observation.bricks_remaining, 1.0);
        assert_eq!(observation.paddle_x, 0.0);
        assert_eq!(observation.ball_x, 0.0);
    }

    #[test]
    fn reset_is_deterministic() {
        let mut env = env();
        let first: Vec<Observation> = (0..30)
            .map(|_| env.step(PaddleAction::Right).observation)
            .collect();

        env.reset();
        let second: Vec<Observation> = (0..30)
            .map(|_| env.step(PaddleAction::Right).observation)
            .collect();

        assert_eq!(first, second, "Episodes should replay identically");
    }

    #[test]
    fn reset_after_game_over_clears_terminal_state() {
        let mut env = env();
        env.app.world_mut().resource_mut::<Lives>().count = 0;
        let result = env.step(PaddleAction::Stay);
        assert!(result.terminated);

        env.reset();
        assert!(!env.terminated());
        assert_eq!(env.state(), GameState::Playing);
    }

    // --- observation ---

    #[test]
    fn observation_is_normalized() {
        let mut env = env();
        for _ in 0..100 {
            let observation = env.step(PaddleAction::Right).observation;
            for value in observation.to_array() {
                assert!((-1.0..=1.0).contains(&value), "Out of range: {value}");
            }
            assert!((0.0..=1.0).contains(&observation.bricks_remaining));
        }
    }

    #[test]
    fn observation_reports_ball_velocity() {
        let env = env();
        let observation = env.observation();
        assert!(observation.ball_velocity_y > 0.0, "Ball starts upward");
        assert!(observation.ball_velocity_x > 0.0);
    }

    #[test]
    fn observation_array_matches_fields() {
        let env = env();
        let observation = env.observation();
        assert_eq!(
            observation.to_array(),
            [
                observation.paddle_x,
                observation.ball_x,
                observation.ball_y,
                observation.ball_velocity_x,
                observation.ball_velocity_y,
                observation.bricks_remaining,
            ]
        );
        assert_eq!(Observation::LEN, 6);
    }

    // --- actions ---

    #[test]
    fn actions_move_the_paddle_in_both_directions() {
        let mut env = env();

        let right = env.step(PaddleAction::Right).observation.paddle_x;
        assert!(right > 0.0, "Right action should move paddle right");

        env.reset();
        let left = env.step(PaddleAction::Left).observation.paddle_x;
        assert!(left < 0.0, "Left action should move paddle left");

        env.reset();
        let stay = env.step(PaddleAction::Stay).observation.paddle_x;
        assert_eq!(stay, 0.0, "Stay action should keep the paddle still");
    }

    #[test]
    fn action_repeat_applies_action_multiple_ticks() {
        let mut single = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 1,
            ..default()
        });
        let mut repeated = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 4,
            ..default()
        });

        let single_x = single.step(PaddleAction::Right).observation.paddle_x;
        let repeated_x = repeated.step(PaddleAction::Right).observation.paddle_x;

        assert!(
            (repeated_x - 4.0 * single_x).abs() < 1e-5,
            "Four repeats should move four times as far"
        );
    }

    #[test]
    fn action_repeat_advances_simulation_multiple_ticks() {
        let mut env = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 3,
            ..default()
        });
        let before = env.observation().ball_y;
        let after = env.step(PaddleAction::Stay).observation.ball_y;
        let one_tick = BALL_SPEED * SIM_DT / (WINDOW_HEIGHT / 2.0);

        assert!(
            (after - before - 3.0 * one_tick).abs() < 1e-4,
            "Three ticks should advance the ball three steps"
        );
    }

    // --- rewards ---

    #[test]
    fn no_reward_without_progress() {
        let mut env = env();
        assert_eq!(env.step(PaddleAction::Stay).reward, 0.0);
    }

    #[test]
    fn destroying_bricks_gives_positive_reward() {
        let mut env = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 1,
            ..default()
        });

        let mut reward_for_first_brick = None;
        for _ in 0..600 {
            let score_before = env.score();
            let result = env.step(PaddleAction::Stay);
            if env.score() > score_before {
                reward_for_first_brick = Some(result.reward);
                break;
            }
            if result.terminated {
                break;
            }
        }

        assert_eq!(
            reward_for_first_brick,
            Some(REWARD_PER_BRICK),
            "Destroying a brick should give exactly one brick reward"
        );
    }

    #[test]
    fn losing_a_life_gives_negative_reward() {
        let mut env = env();

        // Send the ball straight down into the death zone
        let mut lives_before = env.lives();
        let mut reward = 0.0;
        for _ in 0..600 {
            {
                let world = env.app.world_mut();
                let mut query = world.query::<&mut Ball>();
                if let Some(mut ball) = query.iter_mut(world).next() {
                    ball.velocity = Vec2::new(0.0, -BALL_SPEED);
                }
            }
            let result = env.step(PaddleAction::Left);
            if env.lives() < lives_before {
                reward = result.reward;
                break;
            }
            lives_before = env.lives();
        }

        assert!(
            reward <= REWARD_LIFE_LOST,
            "Losing a life should be penalized, got {reward}"
        );
    }

    #[test]
    fn victory_gives_a_bonus_reward() {
        let mut env = env();

        // Clear the board to trigger victory on the next tick
        let world = env.app.world_mut();
        let _ = world.run_system_cached(clear_bricks);
        let result = env.step(PaddleAction::Stay);

        assert!(result.terminated);
        assert_eq!(env.state(), GameState::Victory);
        assert!(result.reward >= REWARD_VICTORY);
    }

    fn clear_bricks(mut commands: Commands, query: Query<Entity, With<Brick>>) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }

    // --- terminal states ---

    #[test]
    fn game_over_terminates_the_episode() {
        let mut env = env();
        env.app.world_mut().resource_mut::<Lives>().count = 0;

        let result = env.step(PaddleAction::Stay);

        assert!(result.terminated);
        assert_eq!(env.state(), GameState::GameOver);
        assert!(result.reward <= REWARD_GAME_OVER);
    }

    #[test]
    fn stepping_a_terminated_episode_is_a_no_op() {
        let mut env = env();
        env.app.world_mut().resource_mut::<Lives>().count = 0;
        env.step(PaddleAction::Stay);

        let before = env.observation();
        let result = env.step(PaddleAction::Right);

        assert!(result.terminated);
        assert_eq!(result.reward, 0.0);
        assert_eq!(result.observation, before);
    }

    // --- randomization ---

    #[test]
    fn randomized_launch_changes_trajectory() {
        let mut fixed = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 1,
            seed: 5,
            randomize_ball_direction: false,
        });
        let mut random = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 1,
            seed: 5,
            randomize_ball_direction: true,
        });

        assert_ne!(
            fixed.step(PaddleAction::Stay).observation.ball_velocity_x,
            random.step(PaddleAction::Stay).observation.ball_velocity_x
        );
    }

    #[test]
    fn randomized_launch_is_reproducible_for_a_seed() {
        let config = EnvConfig {
            action_repeat: 1,
            seed: 11,
            randomize_ball_direction: true,
        };
        let mut a = BreakoutEnv::with_config(config);
        let mut b = BreakoutEnv::with_config(config);

        for _ in 0..20 {
            assert_eq!(
                a.step(PaddleAction::Right).observation,
                b.step(PaddleAction::Right).observation
            );
        }
    }

    #[test]
    fn randomized_launch_varies_between_episodes() {
        let mut env = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 1,
            seed: 3,
            randomize_ball_direction: true,
        });

        let first = env.observation().ball_velocity_x;
        env.reset();
        let second = env.observation().ball_velocity_x;

        assert_ne!(
            first, second,
            "Randomized episodes should not reuse one fixed trajectory"
        );
    }

    #[test]
    fn fixed_launch_is_identical_between_episodes() {
        let mut env = env();
        let first = env.observation();
        env.reset();
        assert_eq!(first, env.observation());
    }

    #[test]
    fn reset_with_seed_restarts_the_random_stream() {
        let mut env = BreakoutEnv::with_config(EnvConfig {
            action_repeat: 1,
            seed: 3,
            randomize_ball_direction: true,
        });

        let first = env.observation();
        env.reset();
        let reseeded = env.reset_with_seed(3);

        assert_eq!(first, reseeded, "Reseeding should replay the same episode");
    }

    // --- headless app ---

    #[test]
    fn headless_app_has_no_window() {
        let mut app = headless_app();
        app.update();

        let mut windows = app.world_mut().query::<&Window>();
        assert_eq!(windows.iter(app.world()).count(), 0);
    }
}
