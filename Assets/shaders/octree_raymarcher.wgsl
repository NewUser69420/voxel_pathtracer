const U32MAX: u32 = 4294967295;
const EPSILON: f32 = 1 * pow(10.0, -6.0);
const SUN: vec3<f32> = vec3<f32>(2048.0, 8192.0, 2048.0);

struct Octree {
    root: array<u32, 3>,
    width: u32,
}

struct Leaf {
    voxel: OctreeVoxel,
    children: array<u32, 8>,
}

struct OctreeVoxel {
    id: u32,
    color: array<f32, 3>,
}

struct ShaderScreen {
    pos: array<f32, 3>,
    rot: array<f32, 3>,
    width: u32,
    height: u32,
    fov: u32,
}

struct Ray {
    start: vec3<f32>,
    direction: vec3<f32>,
}

struct Aabb {
  min: vec3<f32>,
  max: vec3<f32>,
};

struct Emitter {
    position: vec3<f32>,
    strength: f32,
    range: f32,
    falloff: f32,
}

@group(0) @binding(0) var<storage, read> octree: Octree;
@group(0) @binding(1) var<storage, read> leaves: array<Leaf>;
@group(0) @binding(2) var<storage, read> screen: ShaderScreen;
@group(0) @binding(3) var<storage, read> view_distance: u32;
@group(0) @binding(4) var<storage, read> emitters: array<Emitter>;
@group(0) @binding(5) var<storage, read> emitter_num: u32;
@group(0) @binding(6) var texture: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(16, 18, 1)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>){
    let position = vec2<u32>(global_id.x, global_id.y);
    
    var dir = ray_dir_v4(f32(position.x), f32(position.y));
    dir = rotate_at_z(dir, screen.rot[0]);
    dir = rotate_at_x(dir, screen.rot[1]);
    dir = rotate_at_y(dir, screen.rot[2]);

    let pixel_ray = Ray(vec3<f32>(screen.pos[0], screen.pos[1], screen.pos[2]), dir);
    var color = trace_ray(pixel_ray);
    if (position.x > (960 - 5)) && (position.x < (960 + 5)) && (position.y > (540 - 5)) && (position.y < (540 + 5)) {
        color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    textureStore(texture, position, color);
}

fn ray_dir_v4(x: f32, y: f32) -> vec3<f32> {
    let px = x + 0.5;
    let py = y + 0.5;
    let aspect_ratio = f32(screen.width) / f32(screen.height);

    let ray_dir_x = aspect_ratio * ((2.0 * px / f32(screen.width)) - 1.0);
    let ray_dir_y = (2.0 * py / f32(screen.height)) - 1.0;
    let ray_dir_z = 1.0 / tan(radians(f32(screen.fov) / 3.0));

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

fn trace_ray(ray: Ray) -> vec4<f32> {    
    var length = 0.1;
    while (length < f32(view_distance)){
        let photon = at_length(ray, length);

        var root = octree.root;
        var width = octree.width;
        var node = leaves[0];
        var next_index = 0u;
        var exit = 0u;
        while next_index != U32MAX && exit < 100 {
            let i = get_leaf(root, photon);

            node = leaves[next_index];
            next_index = node.children[i];

            root = get_new_root(i, root, width);
            width = width / 2u;
            exit += 1u;
        }
        if node.voxel.id != 0 {
            var color = array<f32, 3>(node.voxel.color[0], node.voxel.color[1], node.voxel.color[2]);
            var rlv = 0.0;
            for (var i = 0u; i < emitter_num; i ++) {
                var ray_direction = photon - emitters[i].position;
                let distance = distance(photon, emitters[i].position);
                if !cast_ray(photon, ray_direction, distance) {
                    //apply shoft light
                    let range_mod = map_range(
                    0.0, emitters[i].range,
                    0.0, 1.0,
                    distance,
                    ) * emitters[i].falloff;
                    rlv += clamp(((1.0 - range_mod) * emitters[i].strength) - rlv, 0.0, 1.0);
                } else {
                    //apply shadow here
                    rlv += clamp(0.1 - rlv, 0.0, 1.0);
                }
            }

            for (var i = 0; i < 3; i++) {
                color[i] = color[i] * rlv;
            }

            return vec4<f32>(color[0], color[1], color[2], 1.0);
        }

        let box = Aabb(vec3<f32>(f32(root[0] - (width / 2)), f32(root[1] - (width / 2)), f32(root[2] - (width / 2))), vec3<f32>(f32(root[0] + (width / 2)), f32(root[1] + (width / 2)), f32(root[2] + (width / 2))));
        let point = get_intersection_box(Ray(photon, ray.direction), box);
        let distance = distance(point, photon);

        length += distance + 0.1;
    }

    //ray hit nothing
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn map_range(a: f32, b: f32, c: f32, d: f32, s: f32) -> f32 {
    return (c + (s - a) * (d - c) / (b - a));
}

fn at_length(ray: Ray, length: f32) -> vec3<f32> {
    let xlen = ray.start.x + ray.direction.x * length;
    let ylen = ray.start.y + ray.direction.y * length;
    let zlen = ray.start.z + ray.direction.z * length;
    return vec3<f32>(xlen, ylen, zlen);
}

fn get_intersection_box(ray: Ray, box: Aabb) -> vec3<f32> {
    var faces = array<Ray, 6>(
        Ray(box.min, vec3<f32>(-1.0, 0.0, 0.0)),
        Ray(box.max, vec3<f32>(1.0, 0.0, 0.0)),
        Ray(box.min, vec3<f32>(0.0, -1.0, 0.0)),
        Ray(box.max, vec3<f32>(0.0, 1.0, 0.0)),
        Ray(box.min, vec3<f32>(0.0, 0.0, -1.0)),
        Ray(box.max, vec3<f32>(0.0, 0.0, 1.0)),
    );
    
    var point = vec3<f32>();
    for (var i = 0; i < 6; i += 1) {
        let temp_point = ray_plane_intersect(ray, faces[i]);
        if temp_point.x != 0.0 && temp_point.y != 0.0 && temp_point.z != 0.0 {
            if distance(ray.start, temp_point) < distance(ray.start, point) {point = temp_point;}
        }
    }

    return point;
}

fn ray_plane_intersect(ray: Ray, plane: Ray) -> vec3<f32> {
    let u = at_length(ray, 1000.0) - ray.start;
    let dot = dot(plane.direction, u);

    if dot > EPSILON {
        let w = ray.start - plane.start;
        let fac = -(dot(plane.direction, w)) / dot;
        let u = u * fac;
        return (ray.start + u);
    } else {
        return vec3<f32>();
    }
}

fn is_within_box(point: vec3<f32>, aabb: Aabb) -> bool {
    return (point.x <= aabb.max.x + 0.3
        && point.x >= aabb.min.x - 0.3
        && point.y <= aabb.max.y + 0.3
        && point.y >= aabb.min.y - 0.3
        && point.z <= aabb.max.z + 0.3
        && point.z >= aabb.min.z - 0.3);
}

fn cast_ray(pos: vec3<f32>, direction: vec3<f32>, ray_length: f32) -> bool {
    let ray = Ray(pos, direction);

    var length = 0.1;
    while (length < ray_length){
        let photon = at_length(ray, length);

        var root = octree.root;
        var width = octree.width;
        var node = leaves[0];
        var next_index = 0u;
        var exit = 0u;
        while next_index != U32MAX && exit < 100 {
            let i = get_leaf(root, photon);

            node = leaves[next_index];
            next_index = node.children[i];

            root = get_new_root(i, root, width);
            width = width / 2u;
            exit += 1u;
        }
        if node.voxel.id != 0 {
            //ray hit something
            return true;
        }

        let box = Aabb(vec3<f32>(f32(root[0] - (width / 2)), f32(root[1] - (width / 2)), f32(root[2] - (width / 2))), vec3<f32>(f32(root[0] + (width / 2)), f32(root[1] + (width / 2)), f32(root[2] + (width / 2))));
        let point = get_intersection_box(Ray(photon, ray.direction), box);
        let distance = distance(point, photon);

        length += distance + (0.1 * pow(length, 0.4));
    }

    //ray hit nothing
    return false;
}

// fn get_shadow(pos: vec3<f32>) -> f32 {
//     let direction = normalize(SUN - pos);
//     let ray = Ray(pos, direction);

//     var length = 0.1;
//     while (length < 64.0){
//         let photon = at_length(ray, length);

//         var root = octree.root;
//         var width = octree.width;
//         var node = leaves[0];
//         var next_index = 0u;
//         var exit = 0u;
//         while next_index != U32MAX && exit < 100 {
//             let i = get_leaf(root, photon);

//             node = leaves[next_index];
//             next_index = node.children[i];

//             root = get_new_root(i, root, width);
//             width = width / 2u;
//             exit += 1u;
//         }
//         if node.voxel.id != 0 {
//             return 0.06;
//         }

//         let box = Aabb(vec3<f32>(f32(root[0] - (width / 2)), f32(root[1] - (width / 2)), f32(root[2] - (width / 2))), vec3<f32>(f32(root[0] + (width / 2)), f32(root[1] + (width / 2)), f32(root[2] + (width / 2))));
//         let point = get_intersection_box(Ray(photon, ray.direction), box);
//         let distance = distance(point, photon);

//         length += distance + 0.5;
//     }

//     //ray hit nothing
//     return 0.0;
// }

fn get_leaf(root: array<u32, 3>, pos: vec3<f32>) -> u32 {
    if pos[0] < f32(root[0]) {
        //left side
        if pos[1] < f32(root[1]) {
            //left bottom
            if pos[2] < f32(root[2]) {
                //left bottom front
                return 2u;
            } else {
                //left bottom back
                return 6u;
            }
        } else {
            //left top
            if pos[2] < f32(root[2]) {
                //left top front
                return 0u;
            } else {
                //left top back
                return 4u;
            }
        }
    } else {
        //right side
        if pos[1] < f32(root[1]) {
            //right bottom
            if pos[2] < f32(root[2]) {
                //right bottom front
                return 3u;
            } else {
                //right bottom back
                return 7u;
            }
        } else {
            //right top
            if pos[2] < f32(root[2]) {
                //right top front
                return 1u;
            } else {
                //right top back
                return 5u;
            }
        }
    }
}

fn get_new_root(idx: u32, old_root: array<u32, 3>, old_width: u32) -> array<u32, 3> {
    var val = old_width / 4u;
    if val < 1u { val = 1u; }

    switch idx {
        case 0u: {
            let x = old_root[0] - val;
            let y = old_root[1] + val;
            let z = old_root[2] - val;

            return array<u32, 3>(x,y,z);
        }
        case 1u: {
            let x = old_root[0] + val;
            let y = old_root[1] + val;
            let z = old_root[2] - val;

            return array<u32, 3>(x,y,z);
        }
        case 2u: {
            let x = old_root[0] - val;
            let y = old_root[1] - val;
            let z = old_root[2] - val;

            return array<u32, 3>(x,y,z);
        }
        case 3u: {
            let x = old_root[0] + val;
            let y = old_root[1] - val;
            let z = old_root[2] - val;

            return array<u32, 3>(x,y,z);
        }
        case 4u: {
            let x = old_root[0] - val;
            let y = old_root[1] + val;
            let z = old_root[2] + val;

            return array<u32, 3>(x,y,z);
        }
        case 5u: {
            let x = old_root[0] + val;
            let y = old_root[1] + val;
            let z = old_root[2] + val;

            return array<u32, 3>(x,y,z);
        }
        case 6u: {
            let x = old_root[0] - val;
            let y = old_root[1] - val;
            let z = old_root[2] + val;

            return array<u32, 3>(x,y,z);
        }
        case 7u: {
            let x = old_root[0] + val;
            let y = old_root[1] - val;
            let z = old_root[2] + val;

            return array<u32, 3>(x,y,z);
        }
        default: {
            return array<u32, 3>(0,0,0);
        }
    }
}