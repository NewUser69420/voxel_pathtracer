use bevy::prelude::*;

use crate::{
    compute::ComputeOctree,
    octree::{get_leaf, get_new_root, Leaf, ShaderOctree},
};

#[derive(Event)]
pub struct CalculateNormalEvent;

#[derive(Resource, Default)]
pub struct TestPosition(pub Vec3);

#[derive(Resource, Default)]
pub struct TestVector(pub Option<Vec3>);

pub fn check_for_perform(
    keys: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<CalculateNormalEvent>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        info!("Calculating normals...");
        event_writer.send(CalculateNormalEvent);
    }
}

pub fn normal_test(
    shader_octree: Res<ComputeOctree>,
    mut event_reader: EventReader<CalculateNormalEvent>,
    mut test_position: ResMut<TestPosition>,
    mut test_direction: ResMut<TestVector>,
) {
    for _ in 0..event_reader.len() {
        event_reader.read();

        if let Some(octree) = &*shader_octree.0.lock().unwrap() {
            let pos = Vec3::new(
                octree.root[0] + 31.0 + 0.2 - 2.0,
                octree.root[1] + 32.0 - 0.3 + 12.0,
                octree.root[2] + 0.2 + 2.0,
            );
            test_position.0 = pos;

            let normal = compute_normal(
                pos,
                &ShaderOctree {
                    root: octree.root,
                    width: octree.width,
                },
                &octree.leaves,
            );

            info!("normal: {}", normal);

            *test_direction = TestVector(Some(normal));
        }
    }

    event_reader.clear();
}

fn compute_normal(vox_pos: Vec3, octree: &ShaderOctree, leaves: &Vec<Leaf>) -> Vec3 {
    let mut normal = Vec3::ZERO;

    let offset_x = Vec3::new(1.0, 0.0, 0.0);
    let offset_y = Vec3::new(0.0, 1.0, 0.0);
    let offset_z = Vec3::new(0.0, 0.0, 1.0);

    let voxel_xp = check_for_voxel(vox_pos + offset_x, octree, leaves) as i32 as f32;
    let voxel_xm = check_for_voxel(vox_pos - offset_x, octree, leaves) as i32 as f32;
    let voxel_yp = check_for_voxel(vox_pos + offset_y, octree, leaves) as i32 as f32;
    let voxel_ym = check_for_voxel(vox_pos - offset_y, octree, leaves) as i32 as f32;
    let voxel_zp = check_for_voxel(vox_pos + offset_z, octree, leaves) as i32 as f32;
    let voxel_zm = check_for_voxel(vox_pos - offset_z, octree, leaves) as i32 as f32;

    info!("xp: {}, xm: {}", voxel_xp, voxel_xm);
    info!("yp: {}, ym: {}", voxel_yp, voxel_ym);
    info!("zp: {}, zm: {}", voxel_zp, voxel_zm);

    normal.x = -voxel_xp + voxel_xm;
    normal.y = -voxel_yp + voxel_ym;
    normal.z = -voxel_zp + voxel_zm;

    if normal.length() > 0.0 {
        return normal.normalize();
    } else {
        return Vec3::ZERO;
    }
}

fn check_for_voxel(pos: Vec3, octree: &ShaderOctree, leaves: &Vec<Leaf>) -> bool {
    let mut root = octree.root;
    let mut width = octree.width;
    let mut node = &leaves[0];
    let mut next_index = 0;
    let mut exit = 0;
    while next_index != u32::MAX && exit < 100 {
        let i = get_leaf(root, [pos.x, pos.y, pos.z]);
        node = &leaves[next_index as usize];
        next_index = node.children[i as usize];
        if next_index != u32::MAX {
            root = get_new_root(i, root, width);
            width = width / 2.0;
        }
        exit += 1;
    }
    if node.voxel.id != 0 {
        return true;
    } else {
        return false;
    }
}

pub fn draw_gizmos(mut gizmos: Gizmos, p: Res<TestPosition>, d: Res<TestVector>) {
    gizmos.sphere(p.0, Quat::IDENTITY, 0.25, Color::srgb(1.0, 0.0, 0.0));

    if d.0.is_some() {
        let d = d.0.unwrap();
        gizmos.ray(p.0, d * 30.0, Color::srgb(0.0, 1.0, 0.0));
    }
}
