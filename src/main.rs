use breakout_rust::components::PaddleAction;
use breakout_rust::environment::{BreakoutEnv, EnvConfig, Observation};
use breakout_rust::game_app;

fn main() {
    if std::env::args().any(|arg| arg == "--headless" || arg == "--train") {
        run_headless();
        return;
    }

    game_app().run();
}

/// Runs the simulation headless (no window, rendering, UI, audio or frame
/// limiter) using a simple scripted policy — a starting point for training.
fn run_headless() {
    let mut env = BreakoutEnv::with_config(EnvConfig {
        randomize_ball_direction: true,
        ..Default::default()
    });

    for episode in 0..HEADLESS_EPISODES {
        let mut observation = env.reset();
        let mut total_reward = 0.0;

        for _ in 0..HEADLESS_MAX_STEPS {
            let result = env.step(follow_ball(observation));
            observation = result.observation;
            total_reward += result.reward;

            if result.terminated {
                break;
            }
        }

        println!(
            "episode {episode}: score={} lives={} reward={total_reward:.1} state={:?}",
            env.score(),
            env.lives(),
            env.state()
        );
    }
}

/// Baseline policy: move the paddle towards the ball.
fn follow_ball(observation: Observation) -> PaddleAction {
    let delta = observation.ball_x - observation.paddle_x;

    if delta < -FOLLOW_DEAD_ZONE {
        PaddleAction::Left
    } else if delta > FOLLOW_DEAD_ZONE {
        PaddleAction::Right
    } else {
        PaddleAction::Stay
    }
}

// Headless run settings
const HEADLESS_EPISODES: u32 = 3;
const HEADLESS_MAX_STEPS: u32 = 20_000;
const FOLLOW_DEAD_ZONE: f32 = 0.01;
