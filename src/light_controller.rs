use bevy::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component, Clone)]
pub struct VoxelLightEmitter {
    pub radius: f32,
    pub strenght: f32,
    pub range: i32,
    pub falloff: f32,
    pub fov: u32,
    pub color: Vec3,
}

#[derive(Component)]
pub struct VariableLight(pub (f32, f32, f32));

pub fn animate_lights(
    mut lights: Query<(&mut VoxelLightEmitter, &mut VariableLight)>,
    time: Res<Time>,
) {
    for (mut light, mut var_light) in lights.iter_mut() {
        if var_light.0 .2 > light.strenght - 0.1 && var_light.0 .2 < light.strenght + 0.1 {
            let new_strength = thread_rng().gen_range(var_light.0 .0..=var_light.0 .1);
            var_light.0 .2 = new_strength;
        }
        light.strenght = light
            .strenght
            .lerp(var_light.0 .2, 0.6 * time.delta_seconds());
    }
}
