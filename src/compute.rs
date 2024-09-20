use crate::{
    entity_spawner::VoxelLightEmitter,
    octree::{Octree, ShaderOctree},
    pre_compute::{RESHIGHT, RESWIDTH},
    world_generator,
};
use bevy::{
    app::{App, Plugin},
    asset::{AssetServer, Handle},
    ecs::{
        schedule::apply_deferred,
        system::{Commands, Res, Resource},
        world::{FromWorld, World},
    },
    log::info,
    math::Vec3,
    prelude::{IntoSystemConfigs, Transform},
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            encase::StorageBuffer, AsBindGroup, BindGroup, BindGroupEntries, BindGroupLayout,
            Buffer, CachedComputePipelineId, CachedPipelineState, ComputePipelineDescriptor,
            PipelineCache, ShaderType,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::Image,
        Render, RenderApp, RenderSet,
    },
};
use bevy::{
    ecs::system::ResMut,
    render::{ExtractSchedule, MainWorld},
};
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};
use wgpu::BindingResource;

#[derive(Resource, ExtractResource, Clone, AsBindGroup)]
pub struct RayTracerBuffers {
    octree: Buffer,
    leaves: Buffer,
    screen: Buffer,
    view_distance: Buffer,
    emitters: Buffer,
    emitter_num: Buffer,
}

#[derive(Resource)]
struct RayTracerBufferBindGroup(BindGroup);

#[derive(Resource, Default)]
pub struct ComputeOctree(pub Arc<Mutex<Option<Octree>>>);

#[derive(Resource, Default)]
pub struct LeafBufferData(pub Arc<Mutex<Vec<u8>>>);

#[derive(Resource, Default)]
struct SerialiseTrigger(Arc<Mutex<bool>>);

#[derive(Resource, Default, Clone, Copy, ShaderType)]
pub struct ShaderScreen {
    pub pos: Vec3,
    pub rot: Vec3,
    pub width: u32,
    pub height: u32,
    pub fov: u32,
}

#[derive(Default, Clone, ShaderType)]
pub struct ShaderEmitter {
    pub position: Vec3,
    pub rotation: Vec3,
    pub radius: f32,
    pub strength: f32,
    pub range: f32,
    pub falloff: f32,
    pub fov: u32,
    pub color: Vec3,
}
impl ShaderEmitter {
    fn from_vl_emitter(value: VoxelLightEmitter, position: Vec3, rotation: Vec3) -> Self {
        ShaderEmitter {
            position: position,
            rotation: rotation,
            radius: value.radius,
            strength: value.strenght,
            range: value.range as f32,
            falloff: value.falloff,
            fov: value.fov,
            color: value.color,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct Emitters(pub Vec<ShaderEmitter>);

#[derive(Resource, ExtractResource, Clone, Default)]
pub struct RayTracerTexture {
    pub texture: Handle<Image>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct RayTraceLabel;

pub struct RayTracerPlugin;
impl Plugin for RayTracerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<RayTracerTexture>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ComputeOctree>()
            .init_resource::<LeafBufferData>()
            .init_resource::<SerialiseTrigger>()
            .init_resource::<ShaderScreen>()
            .init_resource::<Emitters>()
            // .add_systems(ExtractSchedule, get_info.run_if(check_info))
            .add_systems(ExtractSchedule, extract_resources)
            .add_systems(
                Render,
                (
                    update_buffers,
                    apply_deferred,
                    prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
                )
                    .chain(),
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(RayTraceLabel, RayTraceNode::default());
        render_graph.add_node_edge(RayTraceLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world.resource::<RenderDevice>().clone();
        render_app
            .init_resource::<RayTracePipeLine>()
            .insert_resource(RayTracerBuffers {
                octree: setup_octree_buffer(render_device.clone()),
                leaves: setup_leaves_buffer(render_device.clone()),
                screen: setup_screen_buffer(render_device.clone()),
                view_distance: setup_view_distance_buffer(render_device.clone()),
                emitters: setup_emitters_buffer(render_device.clone()),
                emitter_num: setup_emitter_num_buffer(render_device.clone()),
            });
    }
}

fn extract_resources(
    mut world: ResMut<MainWorld>,
    mut octree: ResMut<ComputeOctree>,
    mut screen: ResMut<ShaderScreen>,
    mut emitters: ResMut<Emitters>,
    // render_state: Res<State<RenderingState>>,
) {
    let now = Instant::now();

    match world.resource::<ComputeOctree>().0.try_lock() {
        Ok(mut lock) => {
            if lock.is_some() {
                octree.0 = Arc::new(Mutex::new(lock.take()));
            }
        }
        Err(_) => {}
    }

    let o_screen = world.resource::<ShaderScreen>();
    screen.pos = o_screen.pos;
    screen.rot = o_screen.rot;
    screen.height = o_screen.height;
    screen.width = o_screen.width;
    screen.fov = o_screen.fov;

    emitters.0 = world
        .query::<(&VoxelLightEmitter, &Transform)>()
        .iter(&mut world)
        .map(|v| {
            ShaderEmitter::from_vl_emitter(
                v.0.clone(),
                v.1.translation,
                v.1.rotation.to_euler(bevy::math::EulerRot::XYZ).into(),
            )
        })
        .collect();

    // world
    //     .resource_mut::<NextState<RenderingState>>()
    //     .set(*render_state.get());

    let elapsed = now.elapsed().as_millis();
    if elapsed > 0 {
        info!("extracting resources took: {}", elapsed)
    }
}

fn update_buffers(
    raytracer_buffer: ResMut<RayTracerBuffers>,
    octree: Res<ComputeOctree>,
    leaf_data: Res<LeafBufferData>,
    screen: Res<ShaderScreen>,
    emitters: Res<Emitters>,
    render_queue: Res<RenderQueue>,
    trigger: Res<SerialiseTrigger>,
) {
    let now = Instant::now();

    match octree.0.try_lock() {
        Ok(lock) => {
            let oct_clone = Arc::clone(&octree.0);
            let leaf_clone = Arc::clone(&leaf_data.0);
            let trig_clone = Arc::clone(&trigger.0);
            if !*trigger.0.lock().unwrap() {
                *trigger.0.lock().unwrap() = true;
                thread::spawn(move || {
                    serialise_leaf_data(oct_clone, leaf_clone);
                    *trig_clone.lock().unwrap() = false;
                });
            }
            if lock.is_some() {
                update_octree_buffer(
                    render_queue.clone(),
                    &raytracer_buffer.octree,
                    &ShaderOctree::new(lock.as_ref().unwrap().width, lock.as_ref().unwrap().root),
                );
            }
            update_leaves_buffer(
                render_queue.clone(),
                &raytracer_buffer.leaves,
                Arc::clone(&leaf_data.0),
            );
        }
        Err(_) => {}
    }

    update_screen_buffer(render_queue.clone(), &raytracer_buffer.screen, *screen);

    update_emitters_buffer(
        render_queue.clone(),
        &raytracer_buffer.emitters,
        emitters.0.clone(),
    );

    update_emitter_num_buffer(
        render_queue.clone(),
        &raytracer_buffer.emitter_num,
        emitters.0.len(),
    );

    let elapsed = now.elapsed().as_millis();
    if elapsed > 5 {
        info!("updating buffers took: {}", elapsed)
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<RayTracePipeLine>,
    render_device: Res<RenderDevice>,
    // render_queue: Res<RenderQueue>,
    gpu_images: ResMut<RenderAssets<Image>>,
    raytracer_buffer: Res<RayTracerBuffers>,
    raytracer_texture: Res<RayTracerTexture>,
    // render_image: Res<WorldView>,
) {
    let now = Instant::now();
    let gpu_view = gpu_images.get(raytracer_texture.texture.clone()).unwrap();
    match gpu_view.texture_format {
        wgpu::TextureFormat::Rgba8Unorm => {
            let bind_group = render_device.create_bind_group(
                None,
                &pipeline.texture_bind_group_layout,
                &BindGroupEntries::with_indices((
                    (0, raytracer_buffer.octree.as_entire_buffer_binding()),
                    (1, raytracer_buffer.leaves.as_entire_buffer_binding()),
                    (2, raytracer_buffer.screen.as_entire_buffer_binding()),
                    (3, raytracer_buffer.view_distance.as_entire_buffer_binding()),
                    (4, raytracer_buffer.emitters.as_entire_buffer_binding()),
                    (5, raytracer_buffer.emitter_num.as_entire_buffer_binding()),
                    (6, BindingResource::TextureView(&gpu_view.texture_view)),
                )),
            );
            commands.insert_resource(RayTracerBufferBindGroup(bind_group));
        }
        _ => {
            info!("FAILED TO LOAD TEXTURE");
            return;
        }
    }

    let elapsed = now.elapsed().as_millis();
    if elapsed > 0 {
        info!("preparing bind groups took: {}", elapsed)
    }
}

#[derive(Resource)]
struct RayTracePipeLine {
    texture_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for RayTracePipeLine {
    fn from_world(world: &mut World) -> Self {
        let buffer_bind_group_layout = world.resource::<RenderDevice>().create_bind_group_layout(
            None,
            &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        );
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/octree_raymarcher.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![buffer_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        RayTracePipeLine {
            texture_bind_group_layout: buffer_bind_group_layout,
            update_pipeline,
        }
    }
}

enum RayTraceState {
    Loading,
    Update,
}

struct RayTraceNode {
    state: RayTraceState,
}

impl Default for RayTraceNode {
    fn default() -> Self {
        Self {
            state: RayTraceState::Loading,
        }
    }
}

impl render_graph::Node for RayTraceNode {
    fn update(&mut self, world: &mut World) {
        let now = Instant::now();
        let pipeline = world.resource::<RayTracePipeLine>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            RayTraceState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = RayTraceState::Update;
                }
            }
            RayTraceState::Update => {}
        }
        let elapsed = now.elapsed().as_millis();
        if elapsed > 0 {
            info!("update node took: {}", elapsed)
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let now = Instant::now();
        let texture_bind_group = &world.resource::<RayTracerBufferBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeLine>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&wgpu::ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            RayTraceState::Loading => {}
            RayTraceState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups((RESWIDTH / 16) as u32, (RESHIGHT / 18) as u32, 1);
            }
        }

        let elapsed = now.elapsed().as_millis();
        if elapsed > 0 {
            info!("running node took: {}", elapsed)
        }

        Ok(())
    }
}

fn setup_octree_buffer(render_device: RenderDevice) -> Buffer {
    let mut byte_buffer = Vec::new();
    let mut buffer = StorageBuffer::new(&mut byte_buffer);
    buffer.write(&ShaderOctree::default()).unwrap();
    render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: buffer.into_inner(),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    })

    // render_device.create_buffer(&wgpu::BufferDescriptor {
    //     label: None,
    //     size: 9000000,
    //     usage: wgpu::BufferUsages::STORAGE
    //         | wgpu::BufferUsages::COPY_SRC
    //         | wgpu::BufferUsages::COPY_DST,
    //     mapped_at_creation: false,
    // })
}

fn update_octree_buffer(render_queue: RenderQueue, buffer: &Buffer, octree: &ShaderOctree) {
    let mut byte_buffer = Vec::new();
    let mut temp_buffer = StorageBuffer::new(&mut byte_buffer);
    temp_buffer.write(&octree).unwrap();
    render_queue.write_buffer(buffer, 0, &temp_buffer.into_inner());
}

fn setup_leaves_buffer(render_device: RenderDevice) -> Buffer {
    render_device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 300000000,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn update_leaves_buffer(
    render_queue: RenderQueue,
    buffer: &Buffer,
    leaves: Arc<Mutex<std::vec::Vec<u8>>>,
) {
    match leaves.try_lock() {
        Ok(lock) => {
            render_queue.write_buffer(buffer, 0, &lock);
        }
        Err(_) => {}
    }
}

fn serialise_leaf_data(octree: Arc<Mutex<Option<Octree>>>, storage: Arc<Mutex<std::vec::Vec<u8>>>) {
    match octree.lock().unwrap().clone() {
        Some(octree) => {
            let leaves = &octree.leaves;
            let mut byte_buffer = Vec::new();
            let mut temp_buffer = StorageBuffer::new(&mut byte_buffer);
            temp_buffer.write(&leaves).unwrap();
            *storage.lock().unwrap() = temp_buffer.into_inner().to_vec();
        }
        None => {}
    }
}

fn setup_screen_buffer(render_device: RenderDevice) -> Buffer {
    let mut byte_buffer = Vec::new();
    let mut buffer = StorageBuffer::new(&mut byte_buffer);
    buffer.write(&ShaderScreen::default()).unwrap();
    render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: buffer.into_inner(),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    })
}

fn update_screen_buffer(render_queue: RenderQueue, buffer: &Buffer, screen: ShaderScreen) {
    let mut byte_buffer = Vec::new();
    let mut temp_buffer = StorageBuffer::new(&mut byte_buffer);
    temp_buffer.write(&screen).unwrap();
    render_queue.write_buffer(buffer, 0, temp_buffer.into_inner());
}

fn setup_view_distance_buffer(render_device: RenderDevice) -> Buffer {
    let mut byte_buffer = Vec::new();
    let mut buffer = StorageBuffer::new(&mut byte_buffer);
    buffer.write(&world_generator::VIEWDIST).unwrap();
    render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: buffer.into_inner(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    })
}

fn setup_emitters_buffer(render_device: RenderDevice) -> Buffer {
    render_device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 300000000,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn update_emitters_buffer(
    render_queue: RenderQueue,
    buffer: &Buffer,
    emitters: Vec<ShaderEmitter>,
) {
    let mut byte_buffer = Vec::new();
    let mut temp_buffer = StorageBuffer::new(&mut byte_buffer);
    temp_buffer.write(&emitters).unwrap();
    render_queue.write_buffer(buffer, 0, temp_buffer.into_inner());
}

fn setup_emitter_num_buffer(render_device: RenderDevice) -> Buffer {
    let mut byte_buffer = Vec::new();
    let mut buffer = StorageBuffer::new(&mut byte_buffer);
    buffer.write(&0_u32).unwrap();
    render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: buffer.into_inner(),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    })
}

fn update_emitter_num_buffer(render_queue: RenderQueue, buffer: &Buffer, num: usize) {
    let mut byte_buffer = Vec::new();
    let mut temp_buffer = StorageBuffer::new(&mut byte_buffer);
    temp_buffer.write(&(num as u32)).unwrap();
    render_queue.write_buffer(buffer, 0, temp_buffer.into_inner());
}
