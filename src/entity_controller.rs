use bevy::prelude::*;

use crate::world_generator::{VoxWorld, VoxelEntity};

#[derive(Component)]
pub struct MovingEntity;

pub fn move_entities(
    mut entities: Query<&mut VoxelEntity, With<MovingEntity>>,
    mut angle: Local<f32>,
    vox_world: Res<VoxWorld>,
) {
    let radius = 64.0;

    *angle += 1.0;
    if *angle == 360.0 {
        *angle = 0.0;
    }

    let x = radius * angle.to_radians().cos();
    let y = radius * angle.to_radians().sin();

    for mut entity in entities.iter_mut() {
        entity.transform.translation = Vec3::new(
            vox_world.root[0] as f32 + x,
            vox_world.root[1] as f32,
            vox_world.root[2] as f32 + y,
        );
    }
}
