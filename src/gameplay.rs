use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::collision;
use crate::components::*;
use crate::game;
use crate::movement;
use crate::setup;

/// Schedule that performs a complete, deterministic episode reset.
#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimReset;

/// Simulation-only plugin: state, resources and fixed-timestep gameplay systems.
///
/// Contains no rendering, UI, audio or input, so it can be used both by the
/// rendered game and by the headless training environment.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<Scoreboard>()
            .init_resource::<Lives>()
            .init_resource::<PaddleAction>()
            .init_resource::<BallLaunch>()
            .init_resource::<SimRng>()
            .insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP as f64))
            .add_systems(
                SimReset,
                (
                    setup::despawn_sim_entities,
                    setup::reset_sim_resources,
                    setup::spawn_game,
                )
                    .chain(),
            )
            .add_systems(Startup, run_sim_reset)
            // A fresh board is shown in the menu, and every episode starts from
            // the exact same known state. Resuming from Paused never resets.
            .add_systems(OnEnter(GameState::Menu), run_sim_reset)
            .add_systems(
                OnTransition {
                    exited: GameState::Menu,
                    entered: GameState::Playing,
                },
                run_sim_reset,
            )
            .add_systems(
                FixedUpdate,
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

/// Runs the `SimReset` schedule, restoring a known episode start state.
pub fn run_sim_reset(world: &mut World) {
    world.run_schedule(SimReset);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    fn sim_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, GameplayPlugin));
        app.update();
        app
    }

    // --- SimReset ---

    #[test]
    fn startup_spawns_a_full_board() {
        let mut app = sim_app();

        let mut bricks = app.world_mut().query::<&Brick>();
        assert_eq!(bricks.iter(app.world()).count(), BRICK_ROWS * BRICK_COLS);

        let mut balls = app.world_mut().query::<&Ball>();
        assert_eq!(balls.iter(app.world()).count(), 1);
    }

    #[test]
    fn sim_reset_restores_known_state() {
        let mut app = sim_app();

        // Dirty the world
        app.world_mut().resource_mut::<Scoreboard>().score = 500;
        app.world_mut().resource_mut::<Lives>().count = 1;
        let brick = {
            let mut q = app.world_mut().query::<(Entity, &Brick)>();
            q.iter(app.world()).next().unwrap().0
        };
        app.world_mut().entity_mut(brick).despawn();

        run_sim_reset(app.world_mut());

        assert_eq!(app.world().resource::<Scoreboard>().score, 0);
        assert_eq!(app.world().resource::<Lives>().count, INITIAL_LIVES);

        let mut bricks = app.world_mut().query::<&Brick>();
        assert_eq!(bricks.iter(app.world()).count(), BRICK_ROWS * BRICK_COLS);

        let mut balls = app.world_mut().query::<&Ball>();
        assert_eq!(balls.iter(app.world()).count(), 1);

        let mut paddles = app.world_mut().query::<(&Paddle, &Transform)>();
        let paddle_x = paddles.iter(app.world()).next().unwrap().1.translation.x;
        assert!(paddle_x.abs() < f32::EPSILON);
    }
}
