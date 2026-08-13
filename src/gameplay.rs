use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::components::*;
use crate::{collision, game, movement, setup};

/// Schedule holding the deterministic gameplay simulation.
///
/// It is driven by a fixed timestep when rendered and stepped manually by the
/// environment interface when running headless, so the simulation never depends
/// on real frame time.
#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Simulation;

/// Gameplay simulation without any rendering, UI or input.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }

        app.init_state::<GameState>()
            .init_resource::<Scoreboard>()
            .init_resource::<Lives>()
            .init_resource::<PaddleAction>()
            .init_resource::<BallLaunch>()
            .add_systems(Startup, setup::spawn_game)
            .add_systems(OnEnter(GameState::Playing), setup::reset_ball_and_paddle);

        app.init_schedule(Simulation);
        app.add_systems(
            Simulation,
            (
                movement::move_paddle,
                movement::move_ball,
                collision::ball_collision_walls_and_paddle,
                collision::ball_collision_bricks,
                collision::clamp_ball_to_bounds,
                collision::ball_death_zone,
                game::check_game_over,
                game::check_victory,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Runs one fixed simulation tick.
pub fn run_simulation_tick(world: &mut World) {
    world.run_schedule(Simulation);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, GameplayPlugin));
        app
    }

    // --- GameplayPlugin ---

    #[test]
    fn gameplay_plugin_spawns_the_board() {
        let mut app = sim_app();
        app.update();

        let mut bricks = app.world_mut().query::<&Brick>();
        assert_eq!(bricks.iter(app.world()).count(), BRICK_ROWS * BRICK_COLS);

        let mut paddles = app.world_mut().query::<&Paddle>();
        assert_eq!(paddles.iter(app.world()).count(), 1);
    }

    #[test]
    fn simulation_does_not_run_outside_playing_state() {
        let mut app = sim_app();
        app.update();

        let before = ball_position(&mut app);
        app.world_mut().run_schedule(Simulation);
        assert_eq!(
            before,
            ball_position(&mut app),
            "Menu state should be frozen"
        );
    }

    #[test]
    fn simulation_tick_advances_the_ball_deterministically() {
        let mut app = sim_app();
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        let before = ball_position(&mut app);
        app.world_mut().run_schedule(Simulation);
        let after = ball_position(&mut app);

        assert!(
            (after.y - before.y - BALL_SPEED * SIM_DT).abs() < 0.001,
            "Ball should advance exactly one fixed step"
        );
    }

    fn ball_position(app: &mut App) -> Vec2 {
        let mut q = app.world_mut().query_filtered::<&Transform, With<Ball>>();
        q.iter(app.world())
            .next()
            .map(|t| t.translation.truncate())
            .unwrap()
    }
}
