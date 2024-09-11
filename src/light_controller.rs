use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::entity_spawner::{VariableLight, VoxelLightEmitter};

pub fn animate_lights(
    mut lights: Query<(&mut VoxelLightEmitter, &mut VariableLight)>,
    time: Res<Time>,
) {
    for (mut light, mut var_light) in lights.iter_mut() {
        if var_light.0 .2 == light.strenght {
            let new_strength = thread_rng().gen_range(0.2..1.8);
            var_light.0 .2 = new_strength;
        }
        light.strenght = light
            .strenght
            .lerp(var_light.0 .2, 0.5 * time.delta_seconds());
    }
}
