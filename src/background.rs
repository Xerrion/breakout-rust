use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{Material2d, Material2dPlugin};

use crate::components::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BackgroundMaterial>::default())
            .add_systems(Startup, spawn_background)
            .add_systems(Update, update_time);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BackgroundMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub resolution: Vec2,
    #[uniform(0)]
    pub _padding: f32,
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/background.wgsl".into()
    }
}

fn spawn_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(WINDOW_WIDTH, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(BackgroundMaterial {
            time: 0.0,
            resolution: Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            _padding: 0.0,
        })),
        Transform::from_xyz(0.0, 0.0, -100.0),
    ));
}

fn update_time(time: Res<Time>, mut materials: ResMut<Assets<BackgroundMaterial>>) {
    for (_, material) in materials.iter_mut() {
        material.time = time.elapsed_secs();
    }
}
