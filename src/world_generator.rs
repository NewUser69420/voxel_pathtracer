use std::{
    sync::{Arc, RwLock},
    thread,
    time::Instant,
};

use bevy::{prelude::*, utils::HashMap};
use crossbeam_channel::{unbounded, Receiver, Sender};
use dot_vox::*;
use serde::{Deserialize, Serialize};

use crate::octree::OctreeVoxel;

pub const METER: u32 = 8;
pub const VIEWDIST: u32 = 500;
pub const RENDERDIST: u32 = 500;
pub const W_WIDTH: u32 = 2048;
pub const W_HEIGHT: u32 = 2048;
pub const C_SIZE: u32 = 64;

#[derive(Resource)]
pub struct VoxWorld {
    pub world: Arc<RwLock<Vec<Vec<Vec<Chunk>>>>>,
}
impl Default for VoxWorld {
    fn default() -> Self {
        VoxWorld {
            world: Arc::new(RwLock::new(vec![
                vec![
                    vec![
                        Chunk::default();
                        ((W_WIDTH * 2) / C_SIZE) as usize
                    ];
                    ((W_HEIGHT * 2) / C_SIZE) as usize
                ];
                ((W_WIDTH * 2) / C_SIZE) as usize
            ])),
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    pub voxels: HashMap<[u16; 3], StorageVoxel>,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageVoxel {
    pub id: u8,
    pub color: [u8; 3],
}
impl StorageVoxel {
    pub fn into_normal(self: &Self) -> OctreeVoxel {
        OctreeVoxel {
            id: self.id as u32,
            color: Vec3::new(
                self.color[0] as f32 / 100.0,
                self.color[1] as f32 / 100.0,
                self.color[2] as f32 / 100.0,
            ),
        }
    }
}

#[derive(Resource, Clone)]
pub struct WorldData {
    pub data: Vec<Vec<Vec<Chunk>>>,
}
impl Default for WorldData {
    fn default() -> Self {
        WorldData {
            data: vec![
                vec![
                    vec![Chunk::default(); ((W_WIDTH * 2) / C_SIZE) as usize];
                    ((W_HEIGHT * 2) / C_SIZE) as usize
                ];
                ((W_WIDTH * 2) / C_SIZE) as usize
            ],
        }
    }
}

#[derive(Resource)]
pub struct Channel {
    tx: Sender<WorldData>,
    rx: Receiver<WorldData>,
}

pub fn setup(mut commands: Commands) {
    let (tx, rx) = unbounded();
    commands.insert_resource(Channel { tx: tx, rx: rx });
    commands.insert_resource(VoxWorld::default());
}

pub fn build_world(channel: Res<Channel>) {
    let tx = channel.tx.clone();
    thread::spawn(move || {
        let now = Instant::now();

        let mut world = WorldData::default();
        let world_root = [W_WIDTH; 3];

        let room = load_vox("assets/vox_files/room.vox");
        let room_model = &room.models[2];
        let room_voxels = &room_model.voxels;
        let tv_model = &room.models[1];
        let tv_voxels = &tv_model.voxels;
        let chest_model = &room.models[0];
        let chest_voxels = &chest_model.voxels;
        let palette = &room.palette;

        let mut voxel_count = 0;
        for vox in room_voxels.iter() {
            let x = world_root[0] as i32 + vox.x as i32;
            let y = world_root[1] as i32 + vox.z as i32;
            let z = world_root[2] as i32 + vox.y as i32;

            let (xx, yy, zz) = (
                (x as u32 / C_SIZE),
                (y as u32 / C_SIZE),
                (z as u32 / C_SIZE),
            );
            let vox_color = palette[vox.i as usize];
            let color = get_u8_color(vox_color);
            let id = id_from_color([vox_color.r, vox_color.r, vox_color.b]);
            let chunk = &mut world.data[xx as usize][yy as usize][zz as usize];

            voxel_count += 1;
            chunk.voxels.insert(
                [x as u16, y as u16, z as u16],
                StorageVoxel {
                    id: id,
                    color: [color[0], color[1], color[2]],
                },
            );
        }
        for vox in tv_voxels.iter() {
            let x = world_root[0] as i32 + vox.x as i32 + 30;
            let y = world_root[1] as i32 + vox.z as i32 + 2;
            let z = world_root[2] as i32 + vox.y as i32 + 20;

            let (xx, yy, zz) = (
                (x as u32 / C_SIZE),
                (y as u32 / C_SIZE),
                (z as u32 / C_SIZE),
            );
            let vox_color = palette[vox.i as usize];
            let color = get_u8_color(vox_color);
            let id = id_from_color([vox_color.r, vox_color.r, vox_color.b]);
            let chunk = &mut world.data[xx as usize][yy as usize][zz as usize];

            voxel_count += 1;
            chunk.voxels.insert(
                [x as u16, y as u16, z as u16],
                StorageVoxel {
                    id: id,
                    color: [color[0], color[1], color[2]],
                },
            );
        }
        for vox in chest_voxels.iter() {
            let x = world_root[0] as i32 + vox.x as i32 + 20;
            let y = world_root[1] as i32 + vox.z as i32 + 2;
            let z = world_root[2] as i32 + vox.y as i32 + 30;

            let (xx, yy, zz) = (
                (x as u32 / C_SIZE),
                (y as u32 / C_SIZE),
                (z as u32 / C_SIZE),
            );
            let vox_color = palette[vox.i as usize];
            let color = get_u8_color(vox_color);
            let id = id_from_color([vox_color.r, vox_color.r, vox_color.b]);
            let chunk = &mut world.data[xx as usize][yy as usize][zz as usize];

            voxel_count += 1;
            chunk.voxels.insert(
                [x as u16, y as u16, z as u16],
                StorageVoxel {
                    id: id,
                    color: [color[0], color[1], color[2]],
                },
            );
        }

        let goblin = load_vox("assets/vox_files/goblino.vox");
        let goblin_model = &goblin.models[0];
        let goblin_voxels = &goblin_model.voxels;
        let palette = goblin.palette;
        for vox in goblin_voxels.iter() {
            let x = world_root[0] as i32 + vox.x as i32 + 40;
            let y = world_root[1] as i32 + vox.z as i32 + 2;
            let z = world_root[2] as i32 + vox.y as i32 + 40;

            let (xx, yy, zz) = (
                (x as u32 / C_SIZE),
                (y as u32 / C_SIZE),
                (z as u32 / C_SIZE),
            );
            let vox_color = palette[vox.i as usize];
            let color = get_u8_color(vox_color);
            let id = id_from_color([vox_color.r, vox_color.r, vox_color.b]);
            let chunk = &mut world.data[xx as usize][yy as usize][zz as usize];

            voxel_count += 1;
            chunk.voxels.insert(
                [x as u16, y as u16, z as u16],
                StorageVoxel {
                    id: id,
                    color: [color[0], color[1], color[2]],
                },
            );
        }

        let elapsed = now.elapsed().as_millis();
        if elapsed > 0 {
            info!("Done!, making world took: {} millis", elapsed);
            info!("Voxels: {}", voxel_count);
            // info!(
            //     "midl_chunk_vox_count: {}",
            //     world.data[40][40][40].voxels.iter().len()
            // );
        }

        match tx.send(world) {
            Ok(_) => {}
            Err(err) => info!("Error sending finished octree: {}", err),
        }
    });
}

pub fn _get_id(h: u32) -> u8 {
    if h > W_HEIGHT / 2 {
        return 1;
    } else {
        return 3;
    }
}

pub fn _get_color_by_id(id: u8) -> [u8; 3] {
    let (r, g, b) = match id {
        1 => (50, 50, 50),
        2 => (46, 24, 4),
        3 => (24, 105, 20),
        8 => (255, 231, 22),
        255 => (5, 5, 60),
        _ => (10, 10, 10),
    };

    return get_u8_color(dot_vox::Color { r, g, b, a: 255 });
}

pub fn id_from_color(color: [u8; 3]) -> u8 {
    match color {
        [108, 108, 108] | [90, 90, 90] | [121, 121, 121] => 1,
        [85, 59, 30] | [111, 67, 16] => 2,
        [24, 105, 20] | [0, 131, 15] => 3,
        [75, 26, 11] => 4,
        [69, 40, 13] | [77, 50, 25] | [46, 24, 4] => 5,
        [144, 46, 46] | [59, 59, 59] | [10, 10, 10] => 6,
        [183, 183, 183] | [139, 139, 139] | [54, 54, 54] => 7,
        [255, 231, 22] => 8,
        _ => 1,
    }
}

pub fn get_u8_color(color: dot_vox::Color) -> [u8; 3] {
    let r = map_range((0.0, 255.0), (0.0, 0.2), color.r as f32);
    let g = map_range((0.0, 255.0), (0.0, 0.2), color.g as f32);
    let b = map_range((0.0, 255.0), (0.0, 0.2), color.b as f32);

    return [(r * 100.0) as u8, (g * 100.0) as u8, (b * 100.0) as u8];
}

pub fn map_range(from_range: (f32, f32), to_range: (f32, f32), s: f32) -> f32 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

pub fn receive_world(channel: Res<Channel>, mut world: ResMut<VoxWorld>) {
    if channel.rx.len() > 0 {
        match channel.rx.try_recv() {
            Ok(result) => {
                world.world = Arc::new(RwLock::new(result.data));

                info!("Loaded world!");
            }
            Err(err) => info!("Error receiving world: {}", err),
        }
    }
}

pub fn load_vox(asset: &str) -> DotVoxData {
    let result = load(asset);
    match result {
        Ok(result) => return result,
        Err(err) => {
            panic!("could not load voxel asset: {}, err: {}", asset, err);
        }
    }
}
