use crate::compute;
use crate::player_controller::{PCamera, Player};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use std::time::Instant;

pub const RESWIDTH: i64 = 1920;
pub const RESHIGHT: i64 = 1080;
pub const FOV: i64 = 90;

pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(compute::RayTracerTexture {
        texture: images.add(create_storage_texture((RESWIDTH, RESHIGHT))),
    });
    commands.insert_resource(compute::ShaderScreen::default());
}

pub fn setup_shader_screen(mut shader_screen: ResMut<compute::ShaderScreen>) {
    shader_screen.width = RESWIDTH as u32;
    shader_screen.height = RESHIGHT as u32;
    shader_screen.fov = FOV as u32;
}

pub fn update_shader_screen(
    mut shader_screen: ResMut<compute::ShaderScreen>,
    cam_query: Query<(&GlobalTransform, &Transform, &Camera), (With<PCamera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let now = Instant::now();

    let real_cam = cam_query.single();
    let real_player = player_query.single();
    let player_rotation = real_player.rotation.to_euler(EulerRot::ZXY);
    let cam_rotation = real_cam.1.rotation.to_euler(EulerRot::ZXY);

    shader_screen.pos = [
        real_cam.0.translation().x,
        real_cam.0.translation().y,
        real_cam.0.translation().z,
    ];
    shader_screen.rot = [player_rotation.0, cam_rotation.1, player_rotation.2];

    let elapsed = now.elapsed().as_millis();
    if elapsed > 1 {
        info!("updating player pos took: {} millis", elapsed);
    }
}

fn create_storage_texture((x, y): (i64, i64)) -> Image {
    let mut image = Image::new_fill(
        wgpu::Extent3d {
            width: x as u32,
            height: y as u32,
            depth_or_array_layers: 1,
        },
        wgpu::TextureDimension::D2,
        &[255, 0, 0, 255],
        wgpu::TextureFormat::Rgba8Unorm,
        RenderAssetUsages::all(),
    );
    image.texture_descriptor.usage = wgpu::TextureUsages::COPY_DST
        | wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::STORAGE_BINDING
        | wgpu::TextureUsages::TEXTURE_BINDING;

    image
}
