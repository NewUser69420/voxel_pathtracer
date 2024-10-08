use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use bevy::prelude::*;

use crate::{
    compute::ComputeOctree,
    octree::{get_lod, Octree},
    player_controller::{PCamera, Player},
    world_generator::{
        get_u8_color, id_from_color, StorageVoxel, VoxWorld, VoxelEntity, C_SIZE, ENTITYDRAW,
        RENDERDIST, W_WIDTH,
    },
};

#[derive(Event)]
pub struct GenerateOctreeEvent;

#[derive(Resource)]
pub struct Trigger(pub Arc<Mutex<bool>>);

pub fn setup(mut commands: Commands) {
    let width = W_WIDTH as f32;
    let lock = Arc::new(Mutex::new(Some(Octree::new(width * 2.0, [width; 3]))));
    commands.insert_resource(ComputeOctree(lock));

    let lock = Arc::new(Mutex::new(true));
    commands.insert_resource(Trigger(lock));
}

pub fn _run_octree(mut event_writer: EventWriter<GenerateOctreeEvent>) {
    event_writer.send(GenerateOctreeEvent);
}

pub fn create_octree(
    world: Res<VoxWorld>,
    vox_entities: Query<&VoxelEntity>,
    shader_octree: Res<ComputeOctree>,
    cam_query: Query<&GlobalTransform, (With<PCamera>, Without<Player>)>,
    trigger: Res<Trigger>,
    mut event_reader: EventReader<GenerateOctreeEvent>,
) {
    let now = Instant::now();

    for _ in event_reader.read() {
        let cam_pos = cam_query.single().translation();
        // let cam_forward = cam_query.single().forward();

        match trigger.0.try_lock() {
            Ok(mut lock) => {
                if *lock {
                    *lock = false;

                    let mut vox_entity_data = Vec::new();
                    for vox_entity in vox_entities.iter() {
                        if vox_entity.transform.translation.distance(cam_pos) < ENTITYDRAW as f32 {
                            vox_entity_data.push(vox_entity.clone());
                        }
                    }

                    let trig_clone = Arc::clone(&trigger.0);
                    let world_clone = Arc::clone(&world.world);
                    let octree_clone = Arc::clone(&shader_octree.0);
                    thread::spawn(move || {
                        let noww = Instant::now();

                        let width = W_WIDTH as f32;
                        let root = [width; 3];
                        let mut new_octree = Octree::new(width * 2.0, root);
                        let mut world = world_clone.write().unwrap();

                        let start = -((RENDERDIST / C_SIZE) as i32);
                        let end = (RENDERDIST / C_SIZE) as i32;
                        for cx in start..end {
                            for cy in start..end {
                                for cz in start..end {
                                    let x = ((cam_pos.x as i32 / C_SIZE as i32) + cx)
                                        .min(((W_WIDTH * 2) as i32 / C_SIZE as i32) - 1)
                                        .max(0);
                                    let y = ((cam_pos.y as i32 / C_SIZE as i32) + cy)
                                        .min(((W_WIDTH * 2) as i32 / C_SIZE as i32) - 1)
                                        .max(0);
                                    let z = ((cam_pos.z as i32 / C_SIZE as i32) + cz)
                                        .min(((W_WIDTH * 2) as i32 / C_SIZE as i32) - 1)
                                        .max(0);

                                    // let chunk_pos = Vec3::new(
                                    //     (x * C_SIZE as i32) as f32,
                                    //     (y * C_SIZE as i32) as f32,
                                    //     (z * C_SIZE as i32) as f32,
                                    // );
                                    // let diff_vector = chunk_pos - cam_pos;
                                    // let dot_product = cam_forward.dot(diff_vector);

                                    // if dot_product > 0.0 || chunk_pos.distance(cam_pos) < 128.0 {

                                    // }
                                    let mut counter = 0;
                                    for (vox_pos, vox) in
                                        world[x as usize][y as usize][z as usize].voxels.iter_mut()
                                    {
                                        // let lod = get_lod(
                                        //     Vec3::new(
                                        //         vox_pos[0] as f32,
                                        //         vox_pos[1] as f32,
                                        //         vox_pos[2] as f32,
                                        //     ),
                                        //     cam_pos,
                                        // );

                                        let lod = 1;

                                        match lod {
                                            1 => {
                                                counter = 0;
                                                new_octree.insert(
                                                    [
                                                        vox_pos[0] as f32,
                                                        vox_pos[1] as f32,
                                                        vox_pos[2] as f32,
                                                    ],
                                                    vox.into_normal(),
                                                    lod,
                                                );
                                            }
                                            2 => {
                                                if counter >= 1 {
                                                    counter = 0;
                                                    new_octree.insert(
                                                        [
                                                            vox_pos[0] as f32,
                                                            vox_pos[1] as f32,
                                                            vox_pos[2] as f32,
                                                        ],
                                                        vox.into_normal(),
                                                        lod,
                                                    );
                                                }
                                            }
                                            4 => {
                                                if counter >= 2 {
                                                    counter = 0;
                                                    new_octree.insert(
                                                        [
                                                            vox_pos[0] as f32,
                                                            vox_pos[1] as f32,
                                                            vox_pos[2] as f32,
                                                        ],
                                                        vox.into_normal(),
                                                        lod,
                                                    );
                                                }
                                            }
                                            8 => {
                                                if counter >= 6 {
                                                    counter = 0;
                                                    new_octree.insert(
                                                        [
                                                            vox_pos[0] as f32,
                                                            vox_pos[1] as f32,
                                                            vox_pos[2] as f32,
                                                        ],
                                                        vox.into_normal(),
                                                        lod,
                                                    );
                                                }
                                            }
                                            16 => {
                                                if counter >= 12 {
                                                    counter = 0;
                                                    new_octree.insert(
                                                        [
                                                            vox_pos[0] as f32,
                                                            vox_pos[1] as f32,
                                                            vox_pos[2] as f32,
                                                        ],
                                                        vox.into_normal(),
                                                        lod,
                                                    );
                                                }
                                            }
                                            _ => {}
                                        }
                                        counter += 1;
                                    }
                                }
                            }
                        }

                        for vox_entity in vox_entity_data.iter() {
                            for voxel in vox_entity.voxels.iter() {
                                let x = vox_entity.transform.translation.x + voxel.x as f32;
                                let y = vox_entity.transform.translation.y + voxel.z as f32;
                                let z = vox_entity.transform.translation.z + voxel.y as f32;

                                let vox_color = vox_entity.palette[voxel.i as usize];
                                let color = get_u8_color(vox_color);
                                let id = id_from_color([vox_color.r, vox_color.r, vox_color.b]);

                                let vox = StorageVoxel {
                                    id: id,
                                    color: [color[0], color[1], color[2]],
                                };

                                new_octree.insert(
                                    [x, y, z],
                                    vox.into_normal(),
                                    get_lod(Vec3::new(x, y, z), cam_pos),
                                );
                            }
                        }

                        *trig_clone.lock().unwrap() = true;
                        *octree_clone.lock().unwrap() = Some(new_octree);

                        let elapsed = noww.elapsed().as_millis();
                        if elapsed > 20 {
                            info!("making octree took: {}", elapsed)
                        }
                    });
                }
            }
            Err(_) => {}
        }
    }

    event_reader.clear();

    let elapsed = now.elapsed().as_millis();
    if elapsed > 20 {
        info!("updating octree took: {}", elapsed)
    }
}
