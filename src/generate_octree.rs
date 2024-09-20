use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use bevy::prelude::*;

use crate::{
    compute::ComputeOctree,
    octree::{Octree, _get_lod},
    player_controller::{PCamera, Player},
    world_generator::{VoxWorld, C_SIZE, RENDERDIST, W_HEIGHT, W_WIDTH},
};

#[derive(Resource)]
pub struct Trigger(pub Arc<Mutex<bool>>);

pub fn setup(mut commands: Commands) {
    let width = W_WIDTH;
    let lock = Arc::new(Mutex::new(Some(Octree::new(width * 2, [width; 3]))));
    commands.insert_resource(ComputeOctree(lock));

    let lock = Arc::new(Mutex::new(true));
    commands.insert_resource(Trigger(lock));
}

pub fn update_octree(
    world: Res<VoxWorld>,
    shader_octree: Res<ComputeOctree>,
    cam_query: Query<&GlobalTransform, (With<PCamera>, Without<Player>)>,
    trigger: Res<Trigger>,
) {
    let now = Instant::now();

    let cam_pos = cam_query.single().translation();
    let cam_forward = cam_query.single().forward();

    match trigger.0.try_lock() {
        Ok(mut lock) => {
            if *lock {
                *lock = false;

                // let mut vox_entity_data = HashMap::new();
                // for (t, entity_vox) in vox_entity_query.iter() {
                //     let entity_pos = t.translation;
                //     for voxel in entity_vox.voxels.iter() {
                //         if cam_pos.distance(entity_pos) < RENDERDIST as f32 {
                //             let pos = [
                //                 t.translation.x as u32 + voxel.0[0],
                //                 t.translation.y as u32 + voxel.0[1],
                //                 t.translation.z as u32 + voxel.0[2],
                //             ];
                //             vox_entity_data.insert(pos, voxel.1.into_normal());
                //         }
                //     }
                // }

                let trig_clone = Arc::clone(&trigger.0);
                let world_clone = Arc::clone(&world.world);
                let octree_clone = Arc::clone(&shader_octree.0);
                thread::spawn(move || {
                    let noww = Instant::now();

                    let width = W_WIDTH;
                    let root = [width; 3];
                    let mut new_octree = Octree::new(width * 2, root);
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
                                    .min(((W_HEIGHT * 2) as i32 / C_SIZE as i32) - 1)
                                    .max(0);
                                let z = ((cam_pos.z as i32 / C_SIZE as i32) + cz)
                                    .min(((W_WIDTH * 2) as i32 / C_SIZE as i32) - 1)
                                    .max(0);

                                let chunk_pos = Vec3::new(
                                    (x * C_SIZE as i32) as f32,
                                    (y * C_SIZE as i32) as f32,
                                    (z * C_SIZE as i32) as f32,
                                );
                                let diff_vector = chunk_pos - cam_pos;
                                let dot_product = cam_forward.dot(diff_vector);

                                if dot_product > 0.0 || chunk_pos.distance(cam_pos) < 128.0 {
                                    let mut counter = 0;
                                    for (vox_pos, vox) in
                                        world[x as usize][y as usize][z as usize].voxels.iter_mut()
                                    {
                                        let lod = _get_lod(
                                            Vec3::new(
                                                vox_pos[0] as f32,
                                                vox_pos[1] as f32,
                                                vox_pos[2] as f32,
                                            ),
                                            cam_pos,
                                        );

                                        match lod {
                                            1 => {
                                                counter = 0;
                                                new_octree.insert(
                                                    [
                                                        vox_pos[0] as u32,
                                                        vox_pos[1] as u32,
                                                        vox_pos[2] as u32,
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
                                                            vox_pos[0] as u32,
                                                            vox_pos[1] as u32,
                                                            vox_pos[2] as u32,
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
                                                            vox_pos[0] as u32,
                                                            vox_pos[1] as u32,
                                                            vox_pos[2] as u32,
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
                                                            vox_pos[0] as u32,
                                                            vox_pos[1] as u32,
                                                            vox_pos[2] as u32,
                                                        ],
                                                        vox.into_normal(),
                                                        lod,
                                                    );
                                                }
                                            }
                                            64 => {
                                                if counter >= 50 {
                                                    counter = 0;
                                                    new_octree.insert(
                                                        [
                                                            vox_pos[0] as u32,
                                                            vox_pos[1] as u32,
                                                            vox_pos[2] as u32,
                                                        ],
                                                        vox.into_normal(),
                                                        lod,
                                                    );
                                                }
                                            }
                                            128 => {
                                                if counter >= 100 {
                                                    counter = 0;
                                                    new_octree.insert(
                                                        [
                                                            vox_pos[0] as u32,
                                                            vox_pos[1] as u32,
                                                            vox_pos[2] as u32,
                                                        ],
                                                        vox.into_normal(),
                                                        lod,
                                                    );
                                                }
                                            }
                                            256 => {
                                                if counter >= 256 {
                                                    counter = 0;
                                                    new_octree.insert(
                                                        [
                                                            vox_pos[0] as u32,
                                                            vox_pos[1] as u32,
                                                            vox_pos[2] as u32,
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
                    }

                    // for (vox_pos, vox) in vox_entity_data.iter() {
                    //     let lod = _get_lod(
                    //         Vec3::new(vox_pos[0] as f32, vox_pos[1] as f32, vox_pos[2] as f32),
                    //         cam_pos,
                    //     );

                    //     new_octree.insert(
                    //         [vox_pos[0] as u32, vox_pos[1] as u32, vox_pos[2] as u32],
                    //         *vox,
                    //         lod,
                    //     );
                    // }

                    *trig_clone.lock().unwrap() = true;
                    *octree_clone.lock().unwrap() = Some(new_octree);

                    let elapsed = noww.elapsed().as_millis();
                    if elapsed > 15 {
                        info!("making octree took: {}", elapsed)
                    }
                });
            }
        }
        Err(_) => {}
    }

    let elapsed = now.elapsed().as_millis();
    if elapsed > 30 {
        info!("updating octree took: {}", elapsed)
    }
}
