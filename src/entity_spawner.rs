use bevy::prelude::*;

use crate::world_generator::W_WIDTH;

#[derive(Component, Clone)]
pub struct VoxelLightEmitter {
    pub radius: f32,
    pub strenght: f32,
    pub range: i32,
    pub falloff: f32,
    pub fov: u32,
}

#[derive(Component)]
pub struct VariableLight(pub (f32, f32, f32));

pub fn spawn_point_light(mut commands: Commands) {
    let world_root = [W_WIDTH; 3];

    commands.spawn((
        TransformBundle::from_transform(Transform::from_xyz(
            world_root[0] as f32 + 30.0,
            world_root[1] as f32 + 20.0,
            world_root[2] as f32 + 20.0,
        )),
        VoxelLightEmitter {
            radius: 16.0,
            strenght: 1.8,
            range: 60,
            falloff: 1.2,
            fov: 0,
        },
        VariableLight((0.9, 1.2, 1.0)),
    ));
}
