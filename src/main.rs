use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use compute::RayTracerPlugin;
use generate_octree::{create_octree, GenerateOctreeEvent};
use light_spawner::spawn_point_lights;
use player_controller::{
    initial_grab_cursor, move_player, player_look, spawn_player, InputState, MovementSettings,
};
use pre_compute::{setup_shader_screen, update_shader_screen};
use test::{CalculateNormalEvent, TestPosition, TestVector};
use world_generator::{build_world, receive_world, VoxWorld};

mod compute;
mod generate_octree;
mod light_controller;
mod light_spawner;
mod octree;
mod player_controller;
mod pre_compute;
mod test;
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
        .add_event::<GenerateOctreeEvent>()
        .add_event::<CalculateNormalEvent>()
        .init_resource::<MovementSettings>()
        .init_resource::<InputState>()
        .init_resource::<VoxWorld>()
        .init_resource::<TestPosition>()
        .init_resource::<TestVector>()
        .add_systems(
            Startup,
            (
                pre_compute::setup,
                generate_octree::setup,
                world_generator::setup,
                initial_grab_cursor,
                setup_shader_screen,
                apply_deferred,
                build_world,
                spawn_point_lights,
                spawn_player,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                receive_world,
                move_player,
                player_look,
                update_shader_screen,
                create_octree,
                // test::check_for_perform,
                // test::normal_test,
                // test::draw_gizmos,
            )
                .chain(),
        )
        .run();
}
