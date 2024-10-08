use bevy::{
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    compute::RayTracerTexture,
    light_controller::VoxelLightEmitter,
    pre_compute::{FOV, RESHIGHT, RESWIDTH},
    world_generator::VoxWorld,
};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PCamera;

#[derive(Component)]
pub struct PSprite;

#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub freecam_speed: f32,
}
impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00008,
            freecam_speed: 150.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

pub fn spawn_player(
    mut commands: Commands,
    render_texture: Res<RayTracerTexture>,
    vox_world: Res<VoxWorld>,
) {
    let player = (
        SpatialBundle::from_transform(Transform::from_xyz(
            vox_world.root[0] as f32,
            vox_world.root[1] as f32,
            vox_world.root[2] as f32 + 32.0,
        )),
        VoxelLightEmitter {
            radius: 1.0,
            strenght: 0.9,
            range: 120,
            falloff: 0.8,
            fov: 0,
            color: Vec3::new(1.0, 0.9, 0.8),
        },
        // VariableLight((0.5, 0.9, 0.7)),
        Player,
    );
    let tracer_cam = (
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: (FOV / 2) as f32,
                aspect_ratio: (RESWIDTH / RESHIGHT) as f32,
                ..Default::default()
            }),
            camera: Camera {
                order: 2,
                ..Default::default()
            },
            ..Default::default()
        },
        PCamera,
    );
    let real_cam = Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::Z, Vec3::Y),
        camera: Camera {
            order: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let sprite = (
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(RESWIDTH as f32, RESHIGHT as f32)),
                ..Default::default()
            },
            texture: render_texture.texture.clone(),
            ..Default::default()
        },
        PSprite,
    );

    commands.spawn(player).with_children(|p| {
        p.spawn(tracer_cam);
        p.spawn(real_cam);
        p.spawn(sprite);
    });
}

pub fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for 'initial_grab_cursor'!");
    }
}

fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

pub fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    time: Res<Time>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in player_query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => match key {
                        KeyCode::KeyW => velocity += forward,
                        KeyCode::KeyS => velocity -= forward,
                        KeyCode::KeyA => velocity -= right,
                        KeyCode::KeyD => velocity += right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::ControlLeft => velocity -= Vec3::Y,
                        _ => (),
                    },
                }
            }

            velocity = velocity.normalize_or_zero();

            transform.translation.x += velocity.x * time.delta_seconds() * settings.freecam_speed;
            transform.translation.y += velocity.y * time.delta_seconds() * settings.freecam_speed;
            transform.translation.z += velocity.z * time.delta_seconds() * settings.freecam_speed;
        }
    }
}

pub fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut p_query: Query<&mut Transform, (With<Player>, Without<PCamera>)>,
    mut c_query: Query<&mut Transform, (With<PCamera>, Without<Player>)>,
) {
    if let Ok(window) = primary_window.get_single() {
        let mut c_transform = c_query.single_mut();

        let delta_state = state.as_mut();
        for mut p_transform in p_query.iter_mut() {
            for ev in delta_state.reader_motion.read(&motion) {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let window_scale = window.height().min(window.width());
                        delta_state.pitch -=
                            (settings.sensitivity * ev.delta.y * window_scale).to_radians();

                        delta_state.yaw -=
                            (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }
                delta_state.pitch = delta_state.pitch.clamp(-1.54, 1.54);

                c_transform.rotation = Quat::from_axis_angle(Vec3::X, delta_state.pitch);
                p_transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw);
            }
        }
    } else {
        warn!("Primary window not found for 'freecam_look'!");
    }
}
