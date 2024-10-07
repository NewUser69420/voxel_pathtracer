use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use compute::RayTracerPlugin;
use entity_controller::move_entities;
use generate_octree::create_octree;
use light_controller::animate_lights;
use light_spawner::spawn_point_lights;
use player_controller::{
    initial_grab_cursor, move_player, player_look, spawn_player, InputState, MovementSettings,
};
use pre_compute::{setup_shader_screen, update_shader_screen};
use world_generator::{build_world, receive_world};

mod compute;
mod entity_controller;
mod generate_octree;
mod light_controller;
mod light_spawner;
mod octree;
mod player_controller;
mod pre_compute;
mod world_generator;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "VoxelRayMarcher".into(),
                    resolution: (pre_compute::RESWIDTH as f32, pre_compute::RESHIGHT as f32).into(),
                    mode: WindowMode::Windowed,
                    resizable: false,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            RayTracerPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .init_resource::<MovementSettings>()
        .init_resource::<InputState>()
        .add_systems(
            Startup,
            (
                world_generator::setup,
                pre_compute::setup,
                generate_octree::setup,
                spawn_point_lights,
                initial_grab_cursor,
                setup_shader_screen,
                apply_deferred,
                spawn_player,
                build_world,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                receive_world,
                move_player,
                player_look,
                move_entities,
                animate_lights,
                update_shader_screen,
                create_octree,
            )
                .chain(),
        )
        .run();
}
