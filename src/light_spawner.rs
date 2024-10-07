use bevy::prelude::*;

use crate::{
    light_controller::{VariableLight, VoxelLightEmitter},
    world_generator::VoxWorld,
};

pub fn spawn_point_lights(mut commands: Commands, vox_world: Res<VoxWorld>) {
    //"sun" light
    commands.spawn((
        TransformBundle::from_transform(Transform::from_xyz(
            vox_world.root[0] as f32,
            vox_world.root[1] as f32 + 64.0,
            vox_world.root[2] as f32,
        )),
        VoxelLightEmitter {
            radius: 1.0,
            strenght: 2.3,
            range: 512,
            falloff: 0.5,
            fov: 0,
            color: Vec3::new(1.0, 0.8, 0.8),
        },
        VariableLight((1.8, 2.3, 1.9)),
    ));

    // second sun light
    // commands.spawn((
    //     TransformBundle::from_transform(Transform::from_xyz(
    //         vox_world.root[0] as f32,
    //         vox_world.root[1] as f32 + 500.0,
    //         vox_world.root[2] as f32,
    //     )),
    //     VoxelLightEmitter {
    //         radius: 1.0,
    //         strenght: 3.5,
    //         range: 3000,
    //         falloff: 0.4,
    //         fov: 0,
    //         color: Vec3::new(1.0, 1.0, 1.0),
    //     },
    //     VariableLight((2.4, 3.5, 3.0)),
    // ));

    //green ight
    // commands.spawn((
    //     TransformBundle::from_transform(Transform::from_xyz(
    //         world_root[0] as f32 + 90.0,
    //         world_root[1] as f32 + 10.0,
    //         world_root[2] as f32 + 5.0,
    //     )),
    //     VoxelLightEmitter {
    //         radius: 16.0,
    //         strenght: 1.0,
    //         range: 30,
    //         falloff: 0.8,
    //         fov: 0,
    //         color: Vec3::new(0.0, 1.0, 0.0),
    //     },
    //     VariableLight((0.2, 0.6, 1.0)),
    // ));
}
