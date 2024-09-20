use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use compute::RayTracerPlugin;
use entity_spawner::spawn_point_lights;
use generate_octree::update_octree;
use light_controller::animate_lights;
use player_controller::{
    initial_grab_cursor, move_player, player_look, spawn_player, InputState, MovementSettings,
};
use pre_compute::{setup_shader_screen, update_shader_screen};
use world_generator::{build_world, receive_world};

mod compute;
mod entity_spawner;
mod generate_octree;
mod light_controller;
mod octree;
mod player_controller;
mod pre_compute;
mod world_generator;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "VoxelRayTracer".into(),
                    resolution: (pre_compute::RESWIDTH as f32, pre_compute::RESHIGHT as f32).into(),
                    mode: WindowMode::BorderlessFullscreen,
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
                pre_compute::setup,
                generate_octree::setup,
                world_generator::setup,
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
                animate_lights,
                update_shader_screen,
                update_octree,
            )
                .chain(),
        )
        .run();
}
