use std::{
    sync::{Arc, RwLock},
    thread,
    time::Instant,
};

use bevy::{prelude::*, utils::HashMap};
use crossbeam_channel::{unbounded, Receiver, Sender};
use dot_vox::{load, Model, Rotation, SceneNode, Voxel};
use serde::{Deserialize, Serialize};

use crate::{entity_controller::MovingEntity, octree::OctreeVoxel};

pub const METER: u32 = 8;
pub const VIEWDIST: u32 = 1024;
pub const RENDERDIST: u32 = 1024;
pub const ENTITYDRAW: u32 = 512;
pub const W_WIDTH: u32 = 4096;
pub const W_HEIGHT: u32 = 4096;
pub const C_SIZE: u32 = 64;

#[derive(Resource)]
pub struct VoxWorld {
    pub world: Arc<RwLock<Vec<Vec<Vec<Chunk>>>>>,
    pub root: [u32; 3],
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
            root: [W_WIDTH - (W_WIDTH / 2); 3],
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

#[derive(Component, Clone)]
pub struct VoxelEntity {
    pub transform: Transform,
    pub voxels: Vec<Voxel>,
    pub palette: Vec<dot_vox::Color>,
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

pub fn _spawn_vox_entities(mut commands: Commands, vox_world: Res<VoxWorld>) {
    let sphere_file = load("Assets/vox_files/sphere.vox").unwrap();
    commands.spawn((
        VoxelEntity {
            transform: Transform::from_xyz(
                vox_world.root[0] as f32,
                vox_world.root[1] as f32 + 128.0,
                vox_world.root[2] as f32 + 512.0,
            ),
            voxels: sphere_file.models[0].voxels.clone(),
            palette: sphere_file.palette,
        },
        MovingEntity,
    ));
}

pub fn build_world(channel: Res<Channel>, vox_world: Res<VoxWorld>) {
    let tx = channel.tx.clone();
    let root = vox_world.root;
    thread::spawn(move || {
        let now = Instant::now();

        let mut world = WorldData::default();
        let vox_data = load("Assets/vox_files/sponza.vox").unwrap();
        let palette = vox_data.palette;

        process_scene_node(
            0,
            &vox_data.scenes,
            &vox_data.models,
            Vec3::new(root[0] as f32, root[2] as f32, root[1] as f32),
            Quat::IDENTITY,
            &mut world,
            &palette,
        );

        let elapsed = now.elapsed().as_millis();
        info!("World loading took: {}", elapsed);

        match tx.send(world) {
            Ok(_) => {}
            Err(err) => info!("Error sending finished octree: {}", err),
        }
    });
}

pub fn process_scene_node(
    node: u32,
    scenes: &Vec<SceneNode>,
    r_models: &Vec<Model>,
    root: Vec3,
    rot: Quat,
    world: &mut WorldData,
    palette: &Vec<dot_vox::Color>,
) {
    match &scenes[node as usize] {
        SceneNode::Transform { frames, child, .. } => {
            if frames.len() != 1 {
                unimplemented!("Multiple frame in transform node");
            }

            let frame = &frames[0];

            let this_translation = frame
                .position()
                .map(|position| Vec3 {
                    x: position.x as f32,
                    y: position.y as f32,
                    z: position.z as f32,
                })
                .unwrap_or(Vec3::ZERO);
            let this_rotation = frame
                .orientation()
                .unwrap_or(Rotation::IDENTITY)
                .to_quat_scale()
                .0;

            let rotation = Quat::from_vec4(Vec4::new(
                this_rotation[0],
                this_rotation[1],
                this_rotation[2],
                this_rotation[3],
            ));
            let translation = root + this_translation;

            process_scene_node(
                *child,
                scenes,
                r_models,
                translation,
                rotation,
                world,
                palette,
            );
        }
        SceneNode::Group { children, .. } => {
            // Process each child node recursively
            for child_index in children {
                process_scene_node(*child_index, scenes, r_models, root, rot, world, palette);
            }
        }
        SceneNode::Shape { models, .. } => {
            // Insert voxels using the calculated current position
            insert_voxels(
                &r_models[models[0].model_id as usize],
                root,
                rot,
                palette,
                world,
            );
        }
    }
}

fn insert_voxels(
    model: &Model,
    root: Vec3,
    rotation: Quat,
    palette: &Vec<dot_vox::Color>,
    world: &mut WorldData,
) {
    for vox in model.voxels.iter() {
        let mut voxel_position = Vec3::new(
            (root[0] as f32 - (model.size.x as f32 / 2.0)) + vox.x as f32,
            (root[1] as f32 - (model.size.y as f32 / 2.0)) + vox.y as f32,
            (root[2] as f32 - (model.size.z as f32 / 2.0)) + vox.z as f32,
        );

        let rotation = rotation.to_euler(EulerRot::XZY);
        voxel_position = rotate_around_x(voxel_position, root, rotation.0);
        voxel_position = rotate_around_z(voxel_position, root, rotation.1);
        voxel_position = rotate_around_y(voxel_position, root, rotation.2);

        voxel_position = Vec3::new(voxel_position.x, voxel_position.z, voxel_position.y);

        let (xx, yy, zz) = (
            (voxel_position.x as u32 / C_SIZE),
            (voxel_position.y as u32 / C_SIZE),
            (voxel_position.z as u32 / C_SIZE),
        );
        let vox_color = palette[vox.i as usize];
        let color = get_u8_color(vox_color);
        let id = id_from_color([vox_color.r, vox_color.g, vox_color.b]);
        let chunk = &mut world.data[xx as usize][yy as usize][zz as usize];

        chunk.voxels.insert(
            [
                voxel_position.x as u16,
                voxel_position.y as u16,
                voxel_position.z as u16,
            ],
            StorageVoxel {
                id: id,
                color: [color[0], color[1], color[2]],
            },
        );
    }
}

fn rotate_around_x(p: Vec3, center: Vec3, angle: f32) -> Vec3 {
    let ty = p.y - center.y;
    let tz = p.z - center.z;

    let cos_theta = angle.cos();
    let sin_theta = angle.sin();

    let rotated_y = ty * cos_theta - tz * sin_theta;
    let rotated_z = ty * sin_theta + tz * cos_theta;

    return Vec3::new(p.x, rotated_y + center.y, rotated_z + center.z);
}

fn rotate_around_y(p: Vec3, center: Vec3, angle: f32) -> Vec3 {
    let tx = p.x - center.x;
    let tz = p.z - center.z;

    let cos_theta = angle.cos();
    let sin_theta = angle.sin();

    let rotated_x = tx * cos_theta + tz * sin_theta;
    let rotated_z = -tx * sin_theta + tz * cos_theta;

    return Vec3::new(rotated_x + center.x, p.y, rotated_z + center.z);
}

fn rotate_around_z(p: Vec3, center: Vec3, angle: f32) -> Vec3 {
    let tx = p.x - center.x;
    let ty = p.y - center.y;

    let cos_theta = angle.cos();
    let sin_theta = angle.sin();

    let rotated_x = tx * cos_theta - ty * sin_theta;
    let rotated_y = tx * sin_theta + ty * cos_theta;

    return Vec3::new(rotated_x + center.x, rotated_y + center.y, p.z);
}

pub fn receive_world(channel: Res<Channel>, world: Res<VoxWorld>) {
    for _ in 0..channel.rx.len() {
        if let Ok(result) = channel.rx.try_recv() {
            let world_clone = Arc::clone(&world.world);
            thread::spawn(move || {
                *world_clone.write().unwrap() = result.data;
            });
        }
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
