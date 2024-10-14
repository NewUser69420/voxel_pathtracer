use bevy::{ecs::system::Resource, math::Vec3, render::render_resource::ShaderType};

pub const U32MAX: u32 = 4294967295;

#[derive(ShaderType, Clone, Default, Resource)]
pub struct ShaderOctree {
    pub root: [f32; 3],
    pub width: f32,
}
impl ShaderOctree {
    pub fn new(width: f32, root: [f32; 3]) -> Self {
        Self {
            root: root,
            width: width,
        }
    }
}

#[derive(Default, Clone, Copy, ShaderType, Debug)]
pub struct Leaf {
    pub voxel: OctreeVoxel,
    pub children: [u32; 8],
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
            children: [U32MAX; 8], //put this information into one u32, and extract it back into 8 numbers in the shader
        }
    }
}

#[derive(Default, Clone, Copy, ShaderType, Debug)]
pub struct OctreeVoxel {
    pub color: Vec3,
    pub emission: f32,
    pub light_color: Vec3,
    pub lit: u32,
    pub id: u32,
}
impl OctreeVoxel {
    pub fn empty() -> Self {
        OctreeVoxel {
            color: Vec3::ZERO,
            emission: 0.0,
            light_color: Vec3::ZERO,
            lit: 0,
            id: 0,
        }
    }
}

#[derive(Default, Clone, Debug, Resource)]
pub struct Octree {
    pub root: [f32; 3],
    pub width: f32,
    pub leaves: Vec<Leaf>,
}
impl Octree {
    pub fn new(width: f32, root: [f32; 3]) -> Self {
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

    pub fn insert(self: &mut Self, vox_pos: [f32; 3], voxel: OctreeVoxel, lod: u32) {
        let i = get_leaf(
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
            self.width / 2.0,
            lod,
        );
    }

    pub fn modify(
        self: &mut Self,
        leaf_index: u32,
        vox_pos: [f32; 3],
        voxel: OctreeVoxel,
        root: [f32; 3],
        width: f32,
        lod: u32,
    ) {
        if width as u32 == lod {
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
            let i = get_leaf(
                root,
                [vox_pos[0] as f32, vox_pos[1] as f32, vox_pos[2] as f32],
            );
            let leaf_index = self.leaves[leaf_index as usize].children[i as usize];
            self.modify(
                leaf_index,
                vox_pos,
                voxel,
                get_new_root(i, root, width),
                width / 2.0,
                lod,
            );
        } else {
            let i = get_leaf(
                root,
                [vox_pos[0] as f32, vox_pos[1] as f32, vox_pos[2] as f32],
            );
            let leaf_index = self.leaves[leaf_index as usize].children[i as usize];
            self.modify(
                leaf_index,
                vox_pos,
                voxel,
                get_new_root(i, root, width),
                width / 2.0,
                lod,
            );
        }
    }
}

pub fn get_leaf(root: [f32; 3], pos: [f32; 3]) -> u32 {
    let mut idx: u32 = 0;

    if pos[0] >= root[0] {
        idx |= 1;
    }

    if pos[1] >= root[1] {
        idx |= 0;
    } else {
        idx |= 2;
    }

    if pos[2] >= root[2] {
        idx |= 4;
    }

    return idx;
}

pub fn get_new_root(idx: u32, old_root: [f32; 3], old_width: f32) -> [f32; 3] {
    let x_base = old_root[0];
    let y_base = old_root[1];
    let z_base = old_root[2];

    let val = (old_width / 4.0).max(0.5);

    let x_offset = if (idx & 1) == 0 { -val } else { val };
    let y_offset = if (idx & 2) == 0 { val } else { -val };
    let z_offset = if (idx & 4) == 0 { -val } else { val };

    return [x_base + x_offset, y_base + y_offset, z_base + z_offset];
}

pub fn get_lod(vox_pos: Vec3, cam_pos: Vec3) -> u32 {
    let dist = vox_pos.distance(cam_pos) as u32;
    match dist {
        0..=128 => 1,
        129..=256 => 2,
        257..=1024 => 4,
        1025..=2048 => 8,
        _ => 16,
    }
}
