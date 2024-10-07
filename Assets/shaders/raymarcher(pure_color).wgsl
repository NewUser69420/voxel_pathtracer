const U32MAX: u32 = 4294967295;
const EPSILON: f32 = 1e-6;
const MAXSTEP: u32 = 100;
const SHADOWSOFTNESS: f32 = 1.3;
const LIGHTDIFFUSION: f32 = 30.0;

struct Octree {
    root: array<f32, 3>,
    width: f32,
}

struct Leaf {
    voxel: OctreeVoxel,
    children: array<u32, 8>,
}

struct OctreeVoxel {
    id: u32,
    color: vec3<f32>,
}

struct ShaderScreen {
    pos: vec3<f32>,
    rot: vec3<f32>,
    width: u32,
    height: u32,
    fov: u32,
}

struct AabbRay {
    start: vec3<f32>,
    direction: vec3<f32>,
    inv_direction: vec3<f32>,
}

struct Aabb {
  min: vec3<f32>,
  max: vec3<f32>,
};

struct AabbData {
    aabb: Aabb,
    t: f32,
}

struct Emitter {
    position: vec3<f32>,
    rotation: vec3<f32>,
    radius: f32,
    strength: f32,
    range: f32,
    falloff: f32,
    fov: u32,
    color: vec3<f32>,
}

@group(0) @binding(0) var<storage, read> octree: Octree;
@group(0) @binding(1) var<storage, read_write> leaves: array<Leaf>;
@group(0) @binding(2) var<storage, read> screen: ShaderScreen;
@group(0) @binding(3) var<storage, read> view_distance: u32;
@group(0) @binding(4) var<storage, read> emitters: array<Emitter>;
@group(0) @binding(5) var<storage, read> emitter_num: u32;
@group(0) @binding(6) var texture: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(16, 18, 1)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var position = vec2<u32>(global_id.x, global_id.y);

    var dir = ray_dir_v4(f32(position.x), f32(position.y), screen.fov);
    dir = rotate_at_z(dir, f32(screen.rot[0]));
    dir = rotate_at_x(dir, f32(screen.rot[1]));
    dir = rotate_at_y(dir, f32(screen.rot[2]));

    var r = AabbRay(vec3<f32>(screen.pos[0], screen.pos[1], screen.pos[2]), dir, vec3<f32>());
    r.inv_direction = vec3<f32>(1.0/r.direction.x, 1.0/r.direction.y, 1.0/r.direction.z);
    var color = get_pixel_color(r);
    // if (position.x > (960 - 5)) && (position.x < (960 + 5)) && (position.y > (540 - 5)) && (position.y < (540 + 5)) {
    //     color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    // }

    textureStore(texture, position, color);
}

fn ray_dir_v4(x: f32, y: f32, fov: u32) -> vec3<f32> {
    let px = x + 0.5;
    let py = y + 0.5;
    let aspect_ratio = f32(screen.width) / f32(screen.height);

    let ray_dir_x = aspect_ratio * ((2.0 * px / f32(screen.width)) - 1.0);
    let ray_dir_y = (2.0 * py / f32(screen.height)) - 1.0;
    let ray_dir_z = 1.0 / tan(radians(f32(fov) / 3.0));

    return normalize(vec3<f32>(-ray_dir_x, -ray_dir_y, -ray_dir_z));
}
fn rotate_at_z(vector: vec3<f32>, a: f32) -> vec3<f32> {
    let x = vector.x * cos(a) - vector.y * sin(a);
    let y = vector.x * sin(a) + vector.y * cos(a);
    let z = vector.z;
    return vec3<f32>(x, y, z);
}
fn rotate_at_y(vector: vec3<f32>, b: f32) -> vec3<f32> {
    let x = vector.z * sin(b) + vector.x * cos(b);
    let y = vector.y;
    let z = vector.z * cos(b) - vector.x * sin(b);
    return vec3<f32>(x, y, z);
}
fn rotate_at_x(vector: vec3<f32>, c: f32) -> vec3<f32> {
    let x = vector.x;
    let y = vector.y * cos(c) - vector.z * sin(c);
    let z = vector.y * sin(c) + vector.z * cos(c);
    return vec3<f32>(x, y, z);
}

fn get_pixel_color(r: AabbRay) -> vec4<f32> {
    var length = 0.1;
    var steps = 0u;

    while (length < f32(view_distance) && steps < MAXSTEP) {
        let photon = at_length(r, length);

        var root = octree.root;
        var width = octree.width;
        var node = leaves[0];
        var next_index = 0u;
        var exit = 0u;
        while next_index != U32MAX && exit < MAXSTEP {
            let i = get_leaf(root, photon);

            node = leaves[next_index];
            next_index = node.children[i];

            if next_index != U32MAX {
                root = get_new_root(i, root, width);
                width = width / 2;
            }

            exit += 1u;
        }

        if node.voxel.id != 0 {
            var color = node.voxel.color;

            return vec4<f32>(color[0], color[1], color[2], 1.0);
        }

        //continue to next safe dist
        length += ray_box_intersect(AabbRay(photon, r.direction, r.inv_direction), Aabb(vec3<f32>(root[0] - (width / 2), root[1] - (width / 2), root[2] - (width / 2)), vec3<f32>(root[0] + (width / 2), root[1] + (width / 2), root[2] + (width / 2)))).y + 0.1;
        steps ++;
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn cast_ray(r: AabbRay, range: f32) -> f32 {
    var length = 0.1;
    var steps = 0u;

    while (length < range && steps < MAXSTEP) {
        let photon = at_length(r, length);

        var root = octree.root;
        var width = octree.width;
        var node = leaves[0];
        var next_index = 0u;
        var exit = 0u;
        while next_index != U32MAX && exit < MAXSTEP {
            let i = get_leaf(root, photon);

            node = leaves[next_index];
            next_index = node.children[i];

            if next_index != U32MAX {
                root = get_new_root(i, root, width);
                width = width / 2;
            }

            exit += 1u;
        }

        if node.voxel.id != 0 {
            return length;
        }

        //continue to next safe dist
        length += ray_box_intersect(AabbRay(photon, r.direction, r.inv_direction), Aabb(vec3<f32>(root[0] - (width / 2), root[1] - (width / 2), root[2] - (width / 2)), vec3<f32>(root[0] + (width / 2), root[1] + (width / 2), root[2] + (width / 2)))).y + 0.1;
        steps ++;
    }

    return length;
}

fn ray_box_intersect(r: AabbRay, b: Aabb) -> vec2<f32> {
    var tmin = 0.0;
    var tmax = 1e6;

    let tx1 = (b.min[0] - r.start[0]) * r.inv_direction[0];
    let tx2 = (b.max[0] - r.start[0]) * r.inv_direction[0];

    let ty1 = (b.min[1] - r.start[1]) * r.inv_direction[1];
    let ty2 = (b.max[1] - r.start[1]) * r.inv_direction[1];

    tmin = max(min(tx1, tx2), min(ty1, ty2));
    tmax = min(max(tx1, tx2), max(ty1, ty2));

    let tz1 = (b.min[2] - r.start[2]) * r.inv_direction[2];
    let tz2 = (b.max[2] - r.start[2]) * r.inv_direction[2];

    tmin = max(tmin, min(tz1, tz2));
    tmax = min(tmax, max(tz1, tz2));    
    
    tmin = max(tmin, 0.0);
    return vec2<f32>(tmin, tmax);
}

fn at_length(ray: AabbRay, length: f32) -> vec3<f32> {
    let xlen = ray.start.x + ray.direction.x * length;
    let ylen = ray.start.y + ray.direction.y * length;
    let zlen = ray.start.z + ray.direction.z * length;
    return vec3<f32>(xlen, ylen, zlen);
}

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

fn map_range(a: f32, b: f32, c: f32, d: f32, s: f32) -> f32 {
    return (c + (s - a) * (d - c) / (b - a));
}

fn get_leaf(root: array<f32, 3>, pos: vec3<f32>) -> u32 {
    var idx: u32 = 0u;
    
    if pos[0] >= root[0] {
        idx |= 1u;
    }
    
    if pos[1] >= root[1] {
        idx |= 0u;
    } else {
        idx |= 2u;
    }

    if pos[2] >= root[2] {
        idx |= 4u;
    }
    
    return idx;
}

fn get_new_root(idx: u32, old_root: array<f32, 3>, old_width: f32) -> array<f32, 3> {
    let x_base = old_root[0];
    let y_base = old_root[1];
    let z_base = old_root[2];

    let val = max(old_width / 4.0, 0.5);

    var x_offset = val;
    if (idx & 1u) == 0u {x_offset = -val;}
    var y_offset = -val;
    if (idx & 2u) == 0u {y_offset = val;}
    var z_offset = val;
    if (idx & 4u) == 0u {z_offset = -val;}

    return array<f32, 3>(x_base + x_offset, y_base + y_offset, z_base + z_offset);
}