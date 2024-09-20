use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use bevy::prelude::*;
use rand::{random, thread_rng, Rng};

use crate::{
    compute::ComputeOctree,
    entity_spawner::{VoxelEntity, VoxelLightEmitter},
    world_generator::{map_range, VoxWorld, C_SIZE},
};

#[derive(Resource)]

pub struct StopLight(pub Arc<Mutex<bool>>);

pub fn setup(mut commands: Commands) {
    let lock = Arc::new(Mutex::new(true));
    commands.insert_resource(StopLight(lock));
}

pub fn _cast_light(
    world: Res<VoxWorld>,
    shader_octree: Res<ComputeOctree>,
    vox_entity_query: Query<(&Transform, Option<&VoxelEntity>, Option<&VoxelLightEmitter>)>,
    trigger: Res<StopLight>,
) {
    match trigger.0.try_lock() {
        Ok(mut lock) => {
            if *lock {
                *lock = false;

                let emitters: Vec<((i32, i32, i32), VoxelLightEmitter)> = vox_entity_query
                    .iter()
                    .filter_map(|(t, _, o)| {
                        if o.is_some() {
                            Some((
                                (
                                    t.translation.x as i32,
                                    t.translation.y as i32,
                                    t.translation.z as i32,
                                ),
                                o.unwrap().clone(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();

                let world_clone = Arc::clone(&world.world);
                let octree_clone = Arc::clone(&shader_octree.0);
                let lock_clone = Arc::clone(&trigger.0);
                thread::spawn(move || {
                    let now = Instant::now();

                    for (start, emitter) in emitters.iter() {
                        let points = points_in_sphere(emitter.range, *start);

                        for vox_pos in points.iter() {
                            let x = vox_pos.0 as i32 / C_SIZE as i32;
                            let y = vox_pos.1 as i32 / C_SIZE as i32;
                            let z = vox_pos.2 as i32 / C_SIZE as i32;

                            let chunk = &mut world_clone.write().unwrap()[x as usize][y as usize]
                                [z as usize];
                            if let Some(voxel) = chunk.voxels.get_mut(&[
                                vox_pos.0 as u16,
                                vox_pos.1 as u16,
                                vox_pos.2 as u16,
                            ]) {
                                if let Some(octree) = octree_clone.lock().unwrap().as_mut() {
                                    let hit_pos = octree._cast_ray(
                                        [vox_pos.0 as u32, vox_pos.1 as u32, vox_pos.2 as u32],
                                        Vec3::new(
                                            vox_pos.0 as f32,
                                            vox_pos.1 as f32,
                                            vox_pos.2 as f32,
                                        ) - Vec3::new(
                                            start.0 as f32,
                                            start.1 as f32,
                                            start.2 as f32,
                                        ),
                                    );

                                    if let Some(p) = hit_pos {
                                        let travel_dist =
                                            Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)
                                                .distance(Vec3::new(
                                                    start.0 as f32,
                                                    start.1 as f32,
                                                    start.2 as f32,
                                                ));

                                        let range_mod = map_range(
                                            (0.0, emitter.range as f32),
                                            (0.0, 1.0),
                                            travel_dist,
                                        ) * emitter.falloff;
                                        let lv = (1.0 - range_mod) * emitter.strenght;
                                        let rlv = map_range((0.0, 1.0), (0.0, 255.0), lv) as u8;
                                        voxel.color[3] = rlv;

                                        // let array = [
                                        //     (vox_pos.0 + 1, vox_pos.1, vox_pos.2),
                                        //     (vox_pos.0 - 1, vox_pos.1, vox_pos.2),
                                        //     (vox_pos.0, vox_pos.1 + 1, vox_pos.2),
                                        //     (vox_pos.0, vox_pos.1 - 1, vox_pos.2),
                                        //     (vox_pos.0, vox_pos.1, vox_pos.2 + 1),
                                        //     (vox_pos.0, vox_pos.1, vox_pos.2 - 1),
                                        // ];
                                        // for vox_pos in array.iter() {
                                        //     let x = vox_pos.0 as i32 / C_SIZE as i32;
                                        //     let y = vox_pos.1 as i32 / C_SIZE as i32;
                                        //     let z = vox_pos.2 as i32 / C_SIZE as i32;
                                        //     let chunk = &mut world_clone.write().unwrap()
                                        //         [x as usize]
                                        //         [y as usize]
                                        //         [z as usize];
                                        //     if let Some(voxel) = chunk.voxels.get_mut(&[
                                        //         vox_pos.0 as u16,
                                        //         vox_pos.1 as u16,
                                        //         vox_pos.2 as u16,
                                        //     ]) {
                                        //         voxel.color[3] = (rlv as f32 * 0.8) as u8;
                                        //     }
                                        // }
                                    }
                                }
                            }
                        }
                    }

                    let elapsed = now.elapsed().as_millis();
                    if elapsed > 0 {
                        info!("casting light took: {}", elapsed);
                    }

                    *lock_clone.lock().unwrap() = true;
                });
            }
        }
        Err(_) => {}
    }
}

// pub fn cast_light(
//     world: Res<VoxWorld>,
//     shader_octree: Res<ComputeOctree>,
//     vox_entity_query: Query<(&Transform, Option<&VoxelEntity>, Option<&VoxelLightEmitter>)>,
//     cam_query: Query<&GlobalTransform, (With<PCamera>, Without<Player>)>,
//     trigger: Res<StopLight>,
// ) {
//     match trigger.0.try_lock() {
//         Ok(mut lock) => {
//             if *lock {
//                 *lock = false;

//                 let cam_pos = cam_query.single().translation();
//                 let cam_forward = cam_query.single().forward();

//                 let emitters: Vec<((i32, i32, i32), VoxelLightEmitter)> = vox_entity_query
//                     .iter()
//                     .filter_map(|(t, _, o)| {
//                         if o.is_some() {
//                             Some((
//                                 (
//                                     t.translation.x as i32,
//                                     t.translation.y as i32,
//                                     t.translation.z as i32,
//                                 ),
//                                 o.unwrap().clone(),
//                             ))
//                         } else {
//                             None
//                         }
//                     })
//                     .collect();

//                 let world_clone = Arc::clone(&world.world);
//                 let octree_clone = Arc::clone(&shader_octree.0);
//                 let lock_clone = Arc::clone(&trigger.0);
//                 thread::spawn(move || {
//                     let now = Instant::now();

//                     let start = -(((RENDERDIST / 2) / C_SIZE) as i32);
//                     let end = ((RENDERDIST / 2) / C_SIZE) as i32;
//                     for cx in start..end {
//                         for cy in start..end {
//                             for cz in start..end {
//                                 let x = ((cam_pos.x as i32 / C_SIZE as i32) + cx)
//                                     .min(((W_WIDTH * 2) as i32 / C_SIZE as i32) - 1)
//                                     .max(0);
//                                 let y = ((cam_pos.y as i32 / C_SIZE as i32) + cy)
//                                     .min(((W_HEIGHT * 2) as i32 / C_SIZE as i32) - 1)
//                                     .max(0);
//                                 let z = ((cam_pos.z as i32 / C_SIZE as i32) + cz)
//                                     .min(((W_WIDTH * 2) as i32 / C_SIZE as i32) - 1)
//                                     .max(0);

//                                 let chunk_pos = Vec3::new(
//                                     (x * C_SIZE as i32) as f32,
//                                     (y * C_SIZE as i32) as f32,
//                                     (z * C_SIZE as i32) as f32,
//                                 );
//                                 let diff_vector = chunk_pos - cam_pos;
//                                 let dot_product = cam_forward.dot(diff_vector);

//                                 if dot_product > 0.0 || chunk_pos.distance(cam_pos) < 64.0 {
//                                     for (start, emitter) in emitters.iter() {
//                                         let points = points_in_sphere(emitter.range, *start);

//                                         for vox_pos in points.iter() {
//                                             if let Some(vox) =
//                                                 world_clone.write().unwrap()[x as usize][y as usize]
//                                                     [z as usize]
//                                                     .voxels
//                                                     .get_mut(&[
//                                                         vox_pos.0 as u16,
//                                                         vox_pos.1 as u16,
//                                                         vox_pos.2 as u16,
//                                                     ])
//                                             {
//                                                 if octree_clone.lock().unwrap().is_some() {
//                                                     if octree_clone
//                                                         .lock()
//                                                         .unwrap()
//                                                         .as_mut()
//                                                         .is_some()
//                                                     {
// let hit_pos = octree_clone
//     .lock()
//     .unwrap()
//     .as_mut()
//     .unwrap()
//     ._cast_ray(
//         [
//             vox_pos.0 as u32,
//             vox_pos.1 as u32,
//             vox_pos.2 as u32,
//         ],
//         Vec3::new(
//             vox_pos.0 as f32,
//             vox_pos.1 as f32,
//             vox_pos.2 as f32,
//         ) - Vec3::new(
//             start.0 as f32,
//             start.1 as f32,
//             start.2 as f32,
//         ),
//     );
// if let Some(p) = hit_pos {
// let travel_dist = Vec3::new(
//     p[0] as f32,
//     p[1] as f32,
//     p[2] as f32,
// )
// .distance(Vec3::new(
//     start.0 as f32,
//     start.1 as f32,
//     start.2 as f32,
// ));

// let range_mod = map_range(
//     (0.0, emitter.range as f32),
//     (0.0, 1.0),
//     travel_dist,
// ) * emitter.falloff;
// let lv = (1.0 - range_mod)
//     * emitter.strenght;
// vox.color[3] = map_range(
//     (0.0, 1.0),
//     (0.0, 255.0),
//     lv,
// )
//     as u8;
// }
//                                                     }
//                                                 }
//                                             }
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     }

//                     let elapsed = now.elapsed().as_millis();
//                     if elapsed > 0 {
//                         info!("casting light took: {}", elapsed);
//                     }

//                     *lock_clone.lock().unwrap() = true;
//                 });
//             }
//         }
//         Err(_) => {}
//     }
// }

pub fn points_in_sphere(radius: i32, start: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
    let mut points = Vec::new();

    for x in -radius..=radius {
        let x2 = x * x;
        for y in -radius..=radius {
            let y2 = y * y;
            if x2 + y2 > radius * radius {
                continue;
            }
            for z in -radius..=radius {
                if x2 + y2 + z * z <= radius * radius {
                    points.push((start.0 + x, start.1 + y, start.2 + z));
                }
            }
        }
    }

    points
}
