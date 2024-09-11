use bevy::prelude::*;

use crate::world_generator::{get_u8_color, id_from_color, load_vox, StorageVoxel, W_WIDTH};

#[derive(Component, Clone)]
pub struct VoxelLightEmitter {
    pub strenght: f32,
    pub range: i32,
    pub falloff: f32,
}

#[derive(Component)]
pub struct VariableLight(pub (f32, f32, f32));

#[derive(Component)]
pub struct VoxelEntity {
    pub voxels: Vec<([u32; 3], StorageVoxel)>,
}

pub fn _spawn_tv(mut commands: Commands) {
    let world_root = [W_WIDTH; 3];
    let mut voxels = Vec::new();
    let hallway = load_vox("assets/vox_files/hallway.vox");
    let tv_voxels = &hallway.models[0].voxels;
    let palette = hallway.palette;

    for vox in tv_voxels.iter() {
        let x = vox.x as u32;
        let y = vox.z as u32;
        let z = vox.y as u32;

        let vox_color = palette[vox.i as usize];
        let color = get_u8_color(vox_color);
        let id_from_color = id_from_color([vox_color.r, vox_color.r, vox_color.b]);
        let id_from_color = id_from_color;
        let id = id_from_color;

        voxels.push((
            [x, y, z],
            StorageVoxel {
                id: id,
                color: [color[0], color[1], color[2]],
            },
        ));
    }

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(
            world_root[0] as f32 + 8.0,
            world_root[1] as f32 + 2.0,
            world_root[2] as f32 + 5.0,
        )),
        VoxelEntity { voxels: voxels },
        VoxelLightEmitter {
            strenght: 1.0,
            range: 40,
            falloff: 1.0,
        },
    ));
}

pub fn spawn_point_light(mut commands: Commands) {
    let world_root = [W_WIDTH; 3];

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_xyz(
            world_root[0] as f32 + 30.0,
            world_root[1] as f32 + 20.0,
            world_root[2] as f32 + 20.0,
        )),
        VoxelLightEmitter {
            strenght: 1.5,
            range: 120,
            falloff: 1.0,
        },
        VariableLight((0.9, 1.2, 1.0)),
    ));
}
