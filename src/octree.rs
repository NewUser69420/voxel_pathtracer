use std::{
    f32::EPSILON,
    sync::{Arc, Mutex},
};

use bevy::{ecs::system::Resource, math::Vec3, render::render_resource::ShaderType};

use crate::world_generator::{RENDERDIST, VIEWDIST};

pub const U32MAX: u32 = 4294967295;

#[derive(ShaderType, Clone, Default, Resource)]
pub struct ShaderOctree {
    pub root: [u32; 3],
    pub width: u32,
}
impl ShaderOctree {
    pub fn new(width: u32, root: [u32; 3]) -> Self {
        Self {
            root: root,
            width: width,
        }
    }
}

#[derive(Default, Clone, Debug, Resource)]
pub struct Octree {
    pub root: [u32; 3],
    pub width: u32,
    pub leaves: Vec<Leaf>,
}

#[derive(Default, Clone, Copy, ShaderType, Debug)]
pub struct Leaf {
    pub voxel: OctreeVoxel,
    pub children: [u32; 8],
}

#[derive(Default, Clone, Copy, ShaderType, Debug)]
pub struct OctreeVoxel {
    pub id: u32,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct _Ray {
    pub _start: Vec3,
    pub _direction: Vec3,
}

#[derive(Clone)]
pub struct _Aabb {
    pub _min: Vec3,
    pub _max: Vec3,
}

impl Octree {
    pub fn new(width: u32, root: [u32; 3]) -> Self {
        Octree {
            root: root,
            width: width,
            leaves: vec![
                Leaf::first(),
                Leaf::empty(),
                Leaf::empty(),
                Leaf::empty(),
                Leaf::empty(),
                Leaf::empty(),
                Leaf::empty(),
                Leaf::empty(),
                Leaf::empty(),
            ],
        }
    }

    pub fn insert(self: &mut Self, vox_pos: [u32; 3], voxel: OctreeVoxel, lod: u32) {
        let i = get_leaf_indx(
            self.root,
            [vox_pos[0] as f32, vox_pos[1] as f32, vox_pos[2] as f32],
        );
        let leaf_index = self.leaves[0].children[i as usize];
        //[0(1234), 1(5678), 2(...)]
        self.modify(
            leaf_index,
            vox_pos,
            voxel,
            get_new_root(i, self.root, self.width),
            self.width / 2,
            lod,
        );
    }

    pub fn modify(
        self: &mut Self,
        leaf_index: u32,
        vox_pos: [u32; 3],
        voxel: OctreeVoxel,
        root: [u32; 3],
        width: u32,
        lod: u32,
    ) {
        if width == lod {
            self.leaves[leaf_index as usize].voxel = voxel;
            return;
        }

        //? is there supposed to be an `else if` instead???
        if self.leaves[leaf_index as usize].children[0] == U32MAX {
            let mut base = self.leaves.len();
            for i in 0..8 {
                self.leaves.push(Leaf::empty());
                self.leaves[leaf_index as usize].children[i] = base as u32;
                base += 1;
            }
            let i = get_leaf_indx(
                root,
                [vox_pos[0] as f32, vox_pos[1] as f32, vox_pos[2] as f32],
            );
            let leaf_index = self.leaves[leaf_index as usize].children[i as usize];
            self.modify(
                leaf_index,
                vox_pos,
                voxel,
                get_new_root(i, root, width),
                width / 2,
                lod,
            );
        } else {
            let i = get_leaf_indx(
                root,
                [vox_pos[0] as f32, vox_pos[1] as f32, vox_pos[2] as f32],
            );
            let leaf_index = self.leaves[leaf_index as usize].children[i as usize];
            self.modify(
                leaf_index,
                vox_pos,
                voxel,
                get_new_root(i, root, width),
                width / 2,
                lod,
            );
        }
    }

    pub fn _merge_trees(trees: Vec<Arc<Mutex<Octree>>>, root: [u32; 3]) -> Option<Octree> {
        let width = RENDERDIST;
        let mut new_tree = Octree::new(width * 2, root);
        new_tree.leaves.clear();
        new_tree.leaves.push(Leaf::first());

        for i in 0..8 {
            match trees[i].lock() {
                Ok(tree) => {
                    let length = new_tree.leaves.len() as u32;
                    new_tree.leaves[0].children[i] = length;
                    for ii in 0..tree.leaves.len() {
                        let mut leaf_clone = tree.leaves[ii];
                        if leaf_clone.children[0] != U32MAX {
                            for iii in 0..8 {
                                leaf_clone.children[iii] += length;
                            }
                        }
                        new_tree.leaves.push(leaf_clone);
                    }
                }
                Err(_) => {}
            }
        }

        if new_tree.leaves.len() >= 9 {
            return Some(new_tree);
        } else {
            return None;
        }
    }

    // pub fn _count_voxels(self: &Self) -> (u32, u32, u32) {
    //     let mut i = 0;
    //     let mut ii = 0;
    //     let mut iii = 0;
    //     for leaf in self.leaves.iter() {
    //         if leaf.children[0] == U32MAX {
    //             i += 1;
    //             if leaf.voxel.id != 0 {
    //                 ii += 1;
    //             }
    //         }
    //         iii += 1;
    //     }
    //     (iii, i, ii)
    // }

    // pub fn _debug_voxels(self: &Self) -> Vec<(u32, u8)> {
    //     let mut text = Vec::new();
    //     for leaf in self.leaves.iter() {
    //         text.push((leaf.children[0], leaf.voxel.id));
    //     }
    //     text
    // }

    // pub fn _check_for_voxel(self: &Self, pos: [u32; 3]) -> Option<[f32; 3]> {
    //     let mut root = self.root;
    //     let mut width = self.width;
    //     let mut node = &self.leaves[0];
    //     let mut next_index = 0;
    //     while next_index != U32MAX {
    //         let i = get_leaf_indx(root, [pos[0] as f32, pos[1] as f32, pos[2] as f32]);

    //         node = &self.leaves[next_index as usize];
    //         next_index = node.children[i as usize];

    //         root = get_new_root(i, root, width);
    //         width = width / 2;
    //     }
    //     if node.voxel.id != 0 {
    //         return Some(node.voxel.color);
    //     } else {
    //         return None;
    //     }
    // }

    pub fn _cast_ray(self: &Self, start_pos: [u32; 3], direction: Vec3) -> Option<[u32; 3]> {
        let ray = _Ray {
            _start: Vec3::new(
                start_pos[0] as f32,
                start_pos[1] as f32,
                start_pos[2] as f32,
            ),
            _direction: direction,
        };
        let mut length = 0.1;
        while length < VIEWDIST as f32 {
            let photon = _at_length(ray.clone(), length);

            let mut root = self.root;
            let mut width = self.width;
            let mut node = &self.leaves[0];
            let mut next_index = 0;
            while next_index != U32MAX {
                let i = get_leaf_indx(root, [photon.x, photon.y, photon.z]);

                node = &self.leaves[next_index as usize];
                next_index = node.children[i as usize];

                root = get_new_root(i, root, width);
                width = width / 2;
            }
            if node.voxel.id != 0 {
                return Some([
                    photon.x.round() as u32,
                    photon.y.round() as u32,
                    photon.z.round() as u32,
                ]);
            }

            let aabb = _Aabb {
                _min: Vec3::new(
                    root[0] as f32 - width as f32 / 2.0,
                    root[1] as f32 - width as f32 / 2.0,
                    root[2] as f32 - width as f32 / 2.0,
                ),
                _max: Vec3::new(
                    root[0] as f32 + width as f32 / 2.0,
                    root[1] as f32 + width as f32 / 2.0,
                    root[2] as f32 + width as f32 / 2.0,
                ),
            };

            let point = _get_intersection_box(
                _Ray {
                    _start: photon,
                    _direction: ray._direction,
                },
                aabb,
            );
            let distance = photon.distance(point);

            length += distance + 0.05;
        }

        None
    }
}

impl Leaf {
    pub fn first() -> Self {
        Leaf {
            voxel: OctreeVoxel::empty(),
            children: [1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    pub fn empty() -> Self {
        Leaf {
            voxel: OctreeVoxel::empty(),
            children: [U32MAX; 8],
        }
    }
}

impl OctreeVoxel {
    pub fn empty() -> Self {
        OctreeVoxel {
            id: 0,
            color: Vec3::ZERO,
        }
    }
}

pub fn _get_intersection_box(ray: _Ray, aabb: _Aabb) -> Vec3 {
    let faces = [
        _Ray {
            _start: aabb._min,
            _direction: Vec3::new(-1.0, 0.0, 0.0),
        },
        _Ray {
            _start: aabb._max,
            _direction: Vec3::new(1.0, 0.0, 0.0),
        },
        _Ray {
            _start: aabb._min,
            _direction: Vec3::new(0.0, -1.0, 0.0),
        },
        _Ray {
            _start: aabb._max,
            _direction: Vec3::new(0.0, 1.0, 0.0),
        },
        _Ray {
            _start: aabb._min,
            _direction: Vec3::new(0.0, 0.0, -1.0),
        },
        _Ray {
            _start: aabb._max,
            _direction: Vec3::new(0.0, 0.0, 1.0),
        },
    ];

    let mut point = ray._start;
    for i in 0..6 {
        match _ray_plane_intersect(ray.clone(), faces[i].clone()) {
            Some(result) => {
                if _is_within_box(result, aabb.clone()) {
                    point = result;
                }
            }
            None => {}
        }
    }

    return point;
}

fn _is_within_box(point: Vec3, aabb: _Aabb) -> bool {
    point.x <= aabb._max.x + 0.1
        && point.x >= aabb._min.x - 0.5
        && point.y <= aabb._max.y + 0.5
        && point.y >= aabb._min.y - 0.5
        && point.z <= aabb._max.z + 0.5
        && point.z >= aabb._min.z - 0.5
}

pub fn _ray_plane_intersect(ray: _Ray, plane: _Ray) -> Option<Vec3> {
    let u = _at_length(ray.clone(), 1000.0) - ray._start;
    let dot = plane._direction.dot(u);

    if dot > EPSILON {
        let w = ray._start - plane._start;
        let fac = -(plane._direction.dot(w)) / dot;
        let u = u * fac;
        return Some(ray._start + u);
    } else {
        return None;
    }
}

pub fn get_leaf_indx(root: [u32; 3], pos: [f32; 3]) -> u8 {
    if pos[0] < root[0] as f32 {
        //left side
        if pos[1] < root[1] as f32 {
            //left bottom
            if pos[2] < root[2] as f32 {
                //left bottom front
                return 2;
            } else {
                //left bottom back
                return 6;
            }
        } else {
            //left top
            if pos[2] < root[2] as f32 {
                //left top front
                return 0;
            } else {
                //left top back
                return 4;
            }
        }
    } else {
        //right side
        if pos[1] < root[1] as f32 {
            //right bottom
            if pos[2] < root[2] as f32 {
                //right bottom front
                return 3;
            } else {
                //right bottom back
                return 7;
            }
        } else {
            //right top
            if pos[2] < root[2] as f32 {
                //right top front
                return 1;
            } else {
                //right top back
                return 5;
            }
        }
    }
}

pub fn get_new_root(idx: u8, old_root: [u32; 3], old_width: u32) -> [u32; 3] {
    let val = (old_width / 4).max(1);
    match idx {
        0 => {
            let x = old_root[0] - val;
            let y = old_root[1] + val;
            let z = old_root[2] - val;

            return [x, y, z];
        }
        1 => {
            let x = old_root[0] + val;
            let y = old_root[1] + val;
            let z = old_root[2] - val;

            return [x, y, z];
        }
        2 => {
            let x = old_root[0] - val;
            let y = old_root[1] - val;
            let z = old_root[2] - val;

            return [x, y, z];
        }
        3 => {
            let x = old_root[0] + val;
            let y = old_root[1] - val;
            let z = old_root[2] - val;

            return [x, y, z];
        }
        4 => {
            let x = old_root[0] - val;
            let y = old_root[1] + val;
            let z = old_root[2] + val;

            return [x, y, z];
        }
        5 => {
            let x = old_root[0] + val;
            let y = old_root[1] + val;
            let z = old_root[2] + val;

            return [x, y, z];
        }
        6 => {
            let x = old_root[0] - val;
            let y = old_root[1] - val;
            let z = old_root[2] + val;

            return [x, y, z];
        }
        7 => {
            let x = old_root[0] + val;
            let y = old_root[1] - val;
            let z = old_root[2] + val;

            return [x, y, z];
        }
        _ => return [0, 0, 0],
    }
}

pub fn _get_lod(vox_pos: Vec3, cam_pos: Vec3) -> u32 {
    let dist = vox_pos.distance(cam_pos) as u32;
    match dist {
        0..=128 => 1,
        129..=256 => 2,
        257..=512 => 4,
        513..=1024 => 16,
        1025..=2048 => 64,
        2049..=4096 => 128,
        _ => 256,
    }
}

pub fn _at_length(ray: _Ray, length: f32) -> Vec3 {
    let xlen = ray._start.x + ray._direction.x * length;
    let ylen = ray._start.y + ray._direction.y * length;
    let zlen = ray._start.z + ray._direction.z * length;
    return Vec3::new(xlen, ylen, zlen);
}
