const U32MAX: u32 = 4294967295;
const EPSILON: f32 = 1e-6;
const STACKSIZE: u32 = 50;

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

struct Oct {
    root: array<f32, 3>,
    width: f32,
    leaf_index: u32,
}

struct ShaderScreen {
    pos: vec3<f32>,
    rot: vec3<f32>,
    width: u32,
    height: u32,
    fov: u32,
}

struct Ray {
    start: vec3<f32>,
    direction: vec3<f32>,
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
fn update(@builtin(global_invocation_id) global_id: vec3<u32>){
    var position = vec2<u32>(global_id.x, global_id.y);
    // let value0 = rand(vec2<f32>(f32(arrayLength(&leaves) * position.y), f32(arrayLength(&leaves) * position.x)));
    // let value1 = rand(vec2<f32>(f32(arrayLength(&leaves) * position.x), f32(arrayLength(&leaves) * position.y)));

    var dir = ray_dir_v4(f32(position.x), f32(position.y), screen.fov);
    dir = rotate_at_z(dir, f32(screen.rot[0]));
    dir = rotate_at_x(dir, f32(screen.rot[1]));
    dir = rotate_at_y(dir, f32(screen.rot[2]));

    let pixel_ray = Ray(vec3<f32>(screen.pos[0], screen.pos[1], screen.pos[2]), dir);
    var color = get_pixel_color(pixel_ray);
    if (position.x > (960 - 5)) && (position.x < (960 + 5)) && (position.y > (540 - 5)) && (position.y < (540 + 5)) {
        color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

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

// fn get_pixel_color(ray: Ray) -> vec4<f32> {
//     var length = 0.1;
//     while (length < f32(view_distance)){
//         let photon = at_length(ray, length);

//         var root = octree.root;
//         var width = octree.width;
//         var node = leaves[0];
//         var next_index = 0u;
//         var exit = 0u;
//         while next_index != U32MAX && exit < 50 {
//             let i = get_leaf(root, photon);

//             node = leaves[next_index];
//             next_index = node.children[i];

//             root = get_new_root(i, root, width);
//             width = width / 2;
//             exit += 1u;
//         }
//         if node.voxel.id != 0 {
//             var color = array<f32, 3>(node.voxel.color[0], node.voxel.color[1], node.voxel.color[2]);
            
//             return vec4<f32>(color[0], color[1], color[2], 1.0);
//         }

//         length += abs(sdf_box(photon, vec3<f32>(root[0] - (width / 2), root[1] - (width / 2), root[2] - (width / 2)), vec3<f32>(root[0] + (width / 2), root[1] + (width / 2), root[2] + (width / 2)))) + 0.1;
//     }

//     //ray hit nothing
//     return vec4<f32>(0.0, 0.0, 0.0, 1.0);
// }

// fn get_pixel_color(ray: Ray) -> vec4<f32> {
//     let inv_direction = vec3<f32>(1.0/ray.direction.x, 1.0/ray.direction.y, 1.0/ray.direction.z); //pre calc the inv of ray direction
//     let aabb_ray = AabbRay(ray.start, ray.direction, inv_direction);
//     var found_voxel_color = vec4<f32>(0.0, 0.0, 0.0, 1.0); // Default color for no hit

//     var dist = 1e6;
//     var length = 0.1;
//     while (length < f32(view_distance)){
//         let photon = at_length(ray, length);

//         var root = octree.root;
//         var width = octree.width;
//         var node = leaves[0];
//         var next_index = 0u;
//         var exit = 0u;
//         while next_index != U32MAX && width > 2 && exit < 100 {
//             let i = get_leaf(root, photon);

//             node = leaves[next_index];
//             next_index = node.children[i];

//             root = get_new_root(i, root, width);
//             width = width / 2.0;
//             exit += 1u;
//         }

//         if width <= 2 {
//             var stack: array<Oct, STACKSIZE>;
//             var stack_size = 0u;
//             stack[stack_size] = Oct(root, width, next_index);
//             stack_size ++;
//             while stack_size > 0u && exit < 500 {
//                 stack_size --;
//                 let cur_oct = stack[stack_size];
//                 var root = cur_oct.root;
//                 var width = cur_oct.width;
//                 var node = leaves[cur_oct.leaf_index];

//                 for (var i = 0u; i < 8; i++) {
//                     let r = get_new_root(i, root, width);
//                     let w = width;
//                     let dist_to_box = distance(vec3<f32>(r[0], r[1], r[2]), ray.start);
//                     if dist_to_box < dist {
//                         let box = Aabb(vec3<f32>(r[0] - (w / 2), r[1] - (w / 2), r[2] - (w / 2)), vec3<f32>(r[0] + (w / 2), r[1] + (w / 2), r[2] + (w / 2)));
//                         let intersection = ray_box_intersect(aabb_ray, box);
//                         if intersection.x < intersection.y {
//                             if node.children[i] != U32MAX {
//                                 let child_index = node.children[i];
//                                 let child_node = leaves[child_index];

//                                 if child_node.voxel.id != 0 && intersection.x < dist{
//                                     dist = intersection.x;
//                                     found_voxel_color = vec4<f32>(child_node.voxel.color[0], child_node.voxel.color[1], child_node.voxel.color[2], 1.0);
//                                 } else {
//                                     stack[stack_size] = Oct(get_new_root(i, root, width), width / 2.0, child_index);
//                                     stack_size++;
//                                 }
//                             }
//                         }

//                     }
//                 }
//             }
//         }

//         length += abs(sdf_box(photon, vec3<f32>(root[0] - (width / 2), root[1] - (width / 2), root[2] - (width / 2)), vec3<f32>(root[0] + (width / 2), root[1] + (width / 2), root[2] + (width / 2)))) + 0.01;
//     }

//     //ray hit nothing
//     return found_voxel_color;
// }

fn get_pixel_color(ray: Ray) -> vec4<f32> {
    let inv_direction = vec3<f32>(1.0/ray.direction.x, 1.0/ray.direction.y, 1.0/ray.direction.z); //pre calc the inv of ray direction
    let aabb_ray = AabbRay(ray.start, ray.direction, inv_direction);
    var found_voxel_color = vec4<f32>(0.0, 0.0, 0.0, 1.0); // Default color for no hit

    var dist = 1e6;
    var exit = 0u;

    //prepare the holder of octs
    var stack: array<Oct, STACKSIZE>;
    var stack_size = 0u;
    stack[stack_size] = Oct(octree.root, octree.width, 0u);
    stack_size ++;

    while stack_size > 0u && exit < 500 {
        stack_size --;
        let cur_oct = stack[stack_size];

        var root = cur_oct.root;
        var width = cur_oct.width;
        var node = leaves[cur_oct.leaf_index];

        //get index sorting by checking angle to a static axis of the box being checked, so that it (hopefully) will get the closest one first
        for (var i = 0u; i < 8; i++) {
            let r = get_new_root(i, root, width);
            let w = width;
            
            let dist_to_box = distance(vec3<f32>(r[0], r[1], r[2]), ray.start);
            if dist_to_box < dist {
                let box = Aabb(vec3<f32>(r[0] - (w / 2), r[1] - (w / 2), r[2] - (w / 2)), vec3<f32>(r[0] + (w / 2), r[1] + (w / 2), r[2] + (w / 2)));
                let intersection = ray_box_intersect(aabb_ray, box);
                if intersection.x < intersection.y {
                    if node.children[i] != U32MAX {
                        let child_index = node.children[i];
                        let child_node = leaves[child_index];
    
                        if child_node.voxel.id != 0 && intersection.x < dist{
                            dist = intersection.x;
                            found_voxel_color = vec4<f32>(child_node.voxel.color[0], child_node.voxel.color[1], child_node.voxel.color[2], 1.0);
                        } else {
                            stack[stack_size] = Oct(get_new_root(i, root, width), width / 2.0, child_index);
                            stack_size++;
                        }
                    }
                }
            }
        }

        exit++;
    }

    if exit == 500 {
        found_voxel_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    //return the color it detected
    return found_voxel_color;
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

// fn ray_box_intersect(r: AabbRay, b: Aabb) -> vec2<f32> {
//     var tmin = 0.0;
//     var tmax = 1e6;

//     for (var d = 0; d < 3; d++) {
//         let t1 = (b.min[d] - r.start[d]) * r.inv_direction[d];
//         let t2 = (b.max[d] - r.start[d]) * r.inv_direction[d];

//         tmin = max(tmin, min(min(t1, t2), tmax));
//         tmax = min(tmax, max(max(t1, t2), tmin));
//     }
    
//     return vec2<f32>(tmin, tmax);
// }

// fn multi_ray_box_intersect(r: AabbRay, num_boxes: u32, boxes_in: array<AabbData, 8>) -> array<AabbData, 8> {
//     var boxes = boxes_in;
//     var signs = array<bool, 3>();
//     for (var i = 0; i < 3; i++) {
//         signs[i] = r.inv_direction[i] < 0.0;
//     }

//     for (var i = 0u; i < num_boxes; i++) {
//         let b = boxes[i].aabb;
//         var tmin = 0.0;
//         var tmax = boxes[i].t;

//         for (var d = 0; d < 3; d++) {
//             var bmin = 0.0;
//             var bmax = 1e6;
//             if !signs[d] {
//                 bmin = b.min[d];
//                 bmax = b.max[d];
//             }else{
//                 bmin = b.max[d];
//                 bmax = b.min[d];
//             }
//             let dmin = (bmin - r.start[d]) * r.inv_direction[d];
//             let dmax = (bmax - r.start[d]) * r.inv_direction[d];

//             tmin = max(dmin, tmin);
//             tmax = min(dmax, tmax);
//         }

//         if tmin <= tmax {
//             boxes[i].t = tmin;
//         }
//     }

//     return boxes;
// }

// fn sdf_box(p: vec3<f32>, min: vec3<f32>, max: vec3<f32>) -> f32 {
//     let bc = (min + max) * 0.5;
//     let hs = (max - min) * 0.5;
//     let q = abs(p - bc) - hs;
//     let cq = vec3<f32>(max(q.x, 0.0), max(q.y, 0.0), max(q.z, 0.0));
//     return length(cq) + min(max(q.x, max(q.y, q.z)), 0.0);
// }

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

fn at_length(ray: Ray, length: f32) -> vec3<f32> {
    let xlen = ray.start.x + ray.direction.x * length;
    let ylen = ray.start.y + ray.direction.y * length;
    let zlen = ray.start.z + ray.direction.z * length;
    return vec3<f32>(xlen, ylen, zlen);
}

fn map_range(a: f32, b: f32, c: f32, d: f32, s: f32) -> f32 {
    return (c + (s - a) * (d - c) / (b - a));
}

fn get_leaf(root: array<f32, 3>, pos: vec3<f32>) -> u32 {
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

fn get_new_root(idx: u32, old_root: array<f32, 3>, old_width: f32) -> array<f32, 3> {
    var val = old_width / 4.0;
    if val < 1.0 { val = 1.0; }

    switch idx {
        case 0u: {
            let x = old_root[0] - val;
            let y = old_root[1] + val;
            let z = old_root[2] - val;

            return array<f32, 3>(x,y,z);
        }
        case 1u: {
            let x = old_root[0] + val;
            let y = old_root[1] + val;
            let z = old_root[2] - val;

            return array<f32, 3>(x,y,z);
        }
        case 2u: {
            let x = old_root[0] - val;
            let y = old_root[1] - val;
            let z = old_root[2] - val;

            return array<f32, 3>(x,y,z);
        }
        case 3u: {
            let x = old_root[0] + val;
            let y = old_root[1] - val;
            let z = old_root[2] - val;

            return array<f32, 3>(x,y,z);
        }
        case 4u: {
            let x = old_root[0] - val;
            let y = old_root[1] + val;
            let z = old_root[2] + val;

            return array<f32, 3>(x,y,z);
        }
        case 5u: {
            let x = old_root[0] + val;
            let y = old_root[1] + val;
            let z = old_root[2] + val;

            return array<f32, 3>(x,y,z);
        }
        case 6u: {
            let x = old_root[0] - val;
            let y = old_root[1] - val;
            let z = old_root[2] + val;

            return array<f32, 3>(x,y,z);
        }
        case 7u: {
            let x = old_root[0] + val;
            let y = old_root[1] - val;
            let z = old_root[2] + val;

            return array<f32, 3>(x,y,z);
        }
        default: {
            return array<f32, 3>(0,0,0);
        }
    }
}

// fn trace_ray(ray: Ray) -> vec4<f32> {    
//     let inv_direction = vec3<f32>(1.0/ray.direction.x, 1.0/ray.direction.y, 1.0/ray.direction.z);

//     var length = 0.1;
//     while (length < f32(view_distance)){
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
//             var color = vec3<f32>(node.voxel.color[0], node.voxel.color[1], node.voxel.color[2]);
            // var shadow = 0.0;
            // var intensity = 0.0;
            // //do light : shadow
            // for (var i = 0u; i < emitter_num; i++ ) {
            //     let light = emitters[i];

            //     let dir = light.position - photon;
            //     let ray = Ray(photon, dir);

            //     let dist_to_light = distance(photon, light.position);
            //     let dist_to_hit = cast_ray(ray, light.range);

            //     if dist_to_hit > dist_to_light - light.radius {
            //         let range_mod = map_range(
            //         0.0, light.range,
            //         0.0, 1.0,
            //         round(dist_to_light)
            //         ) * light.falloff;
            //         intensity = (1.0 - range_mod) * light.strength;
            //         shadow = max(shadow, intensity);
            //     }

            //     color *= (light.color - vec3<f32>(1.0, 1.0, 1.0)) * intensity + vec3<f32>(1.0, 1.0, 1.0);
            // }

            // for (var j = 0; j < 3; j++) {
            //     color[j] = color[j] * shadow;
            // }

//             return vec4<f32>(color[0], color[1], color[2], 1.0);
//         }

//         let box = Aabb(vec3<f32>(f32(root[0] - (width / 2)), f32(root[1] - (width / 2)), f32(root[2] - (width / 2))), vec3<f32>(f32(root[0] + (width / 2)), f32(root[1] + (width / 2)), f32(root[2] + (width / 2))));
//         let aabbray = AabbRay(ray.start, ray.direction, inv_direction);
//         let distance = ray_box_intersect(aabbray, box);

//         length += distance + 0.01;
//     }

//     //ray hit nothing
//     return vec4<f32>(0.0, 0.0, 0.0, 1.0);
// }

// fn cast_ray(ray: Ray, range: f32) -> f32 {
//     let inv_direction = vec3<f32>(1.0 / ray.direction.x, 1.0 / ray.direction.y, 1.0 / ray.direction.z);

//     var root = octree.root;
//     var width = octree.width;
//     var node = leaves[0];
//     var next_index = 0u;
//     var exit = 0u;
//     while next_index != U32MAX && exit < 100 {
//         for (var i = 0u; i < 8; i++) {
//             let r = get_new_root(i, root, width);
//             let w = width / 2u;

//             let box = Aabb(vec3<f32>(f32(r[0] - (w / 2)), f32(r[1] - (w / 2)), f32(r[2] - (w / 2))), vec3<f32>(f32(r[0] + (w / 2)), f32(r[1] + (w / 2)), f32(r[2] + (w / 2))));
//             let aabb_ray = AabbRay(ray.start, ray.direction, inv_direction);
//             if ray_box_intersect(aabb_ray, box) {
//                 node = leaves[next_index];
//                 next_index = node.children[i];

//                 root = get_new_root(i, root, width);
//                 width = width / 2u;
//                 break;
//             }
//         }
//         exit += 1u;
//     }
//     if node.voxel.id != 0 {
//         return distance(ray.start, vec3<f32>(f32(root[0]), f32(root[1]), f32(root[2])));
//     }

//     //ray hit nothing
//     return distance(ray.start, at_length(ray, range));
// }

//fn ray_box_intersect(r: AabbRay, b: Aabb) -> bool {
//     var tmin = 0.0;
//     var tmax = 1e6;
//     var tymin = 0.0;
//     var tymax = 1e6;
//     var tzmin = 0.0;
//     var tzmax = 1e6;

//     //do x
//     if !r.sign[0] {
//         tmin = (b.min.x - r.start.x) * r.inv_direction.x;
//         tmax = (b.max.x - r.start.x) * r.inv_direction.x;
//     }else{
//         tmin = (b.max.x - r.start.x) * r.inv_direction.x;
//         tmax = (b.min.x - r.start.x) * r.inv_direction.x;
//     }
//     //do y
//     if !r.sign[1] {
//         tymin = (b.min.y - r.start.y) * r.inv_direction.y;
//         tymax = (b.max.y - r.start.y) * r.inv_direction.y;
//     }else{
//         tymin = (b.max.y - r.start.y) * r.inv_direction.y;
//         tymax = (b.min.y - r.start.y) * r.inv_direction.y;
//     }

//     if (tmin > tymax || tymin > tmax) {
//         return false;
//     }

//     if tymin > tmin {
//         tmin = tymin;
//     }
//     if tymax < tmax {
//         tmax = tymax;
//     }

//     //do z
//     if !r.sign[2] {
//         tzmin = (b.min.z - r.start.z) * r.inv_direction.z;
//         tzmax = (b.max.z - r.start.z) * r.inv_direction.z;
//     }else{
//         tzmin = (b.max.z - r.start.z) * r.inv_direction.z;
//         tzmax = (b.min.z - r.start.z) * r.inv_direction.z;
//     }

//     if (tmin > tzmax || tzmin > tmax) {
//         return false;
//     }

//     if tzmin > tmin {
//         tmin = tzmin;
//     }
//     if tzmax < tmax {
//         tmax = tzmax;
//     }

//     return true;
// }