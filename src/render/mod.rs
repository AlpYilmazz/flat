use std::collections::HashMap;

use bevy_app::{App, CoreStage, Plugin, StartupStage};
use bevy_asset::{AddAsset, Assets, Handle, HandleId};
use bevy_ecs::{
    prelude::{Bundle, Component, Entity, EventReader, EventWriter},
    schedule::{StageLabel, SystemStage},
    system::{Commands, Query, Res, ResMut},
};
use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use winit::dpi::PhysicalSize;

use crate::{
    shaders::{ShaderInstance, TestWgsl},
    texture,
    transform::{GlobalTransform, Transform},
    util::{store, store_many, EngineDefault, Primary, PrimaryEntity, Refer, ReferMany, Store},
    window::{
        events::{CreateWindow, WindowCreated, WindowResized},
        WindowDescriptor, WindowId, Windows, WinitWindows,
    },
};

use self::{
    camera::{Camera, PerspectiveCameraBundle, PerspectiveProjection},
    mesh::Mesh,
    resource::shader::{ShaderSource, ShaderSourceLoader, Shaders},
    resource::{
        buffer::Vertex,
        pipeline::RenderPipeline,
        uniform::{HandleGpuUniform, Uniform},
    },
    system::{AddRenderSystem, RenderAsset},
};

pub mod camera;
pub mod mesh;
pub mod mesh_bevy;
pub mod resource;
pub mod system;

#[derive(StageLabel)]
pub enum RenderStage {
    Prepare,
    Extract,
    Render,
}

pub struct FlatRenderPlugin;
impl Plugin for FlatRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::Last,
            RenderStage::Prepare,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RenderStage::Prepare,
            RenderStage::Extract,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RenderStage::Extract,
            RenderStage::Render,
            SystemStage::parallel(),
        )
        .init_resource::<Surfaces>()
        .init_resource::<Store<RenderPipeline>>()
        .init_resource::<Store<wgpu::BindGroup>>()
        .init_resource::<Shaders>()
        .add_asset_loader(ShaderSourceLoader)
        .add_asset::<ShaderSource>()
        .add_system_to_stage(CoreStage::PreUpdate, create_surface_system)
        .add_system_to_stage(RenderStage::Prepare, reconfigure_surface_system)
        .add_render_system::<Mesh<Vertex>>();

        create_wgpu_resources(app);

        app.add_startup_system_to_stage(StartupStage::PreStartup, test_create_primary_camera)
            .add_startup_system(test_create_render_entity)
            .add_startup_system(test_create_more_windows);
    }
}

#[derive(Component)]
pub struct InstanceData(wgpu::Buffer, u32);

pub struct DepthTexture(texture::Texture);

pub struct SurfaceKit {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(Default)]
pub struct Surfaces(pub HashMap<WindowId, SurfaceKit>);

#[derive(Bundle)]
pub struct RenderEntityBundle<T: RenderAsset> {
    pub pipeline: Refer<RenderPipeline>,
    pub bind_groups: ReferMany<wgpu::BindGroup>,
    pub render_asset: Handle<T>,
}

///
/// Creates wgpu Instance, Device and Queue as World Resources.
///
/// Creates wpgu Surfaces for initial windows.
///
pub fn create_wgpu_resources(app: &mut App) {
    let winit_windows = app.world.get_resource::<WinitWindows>().unwrap();
    let primary_window = winit_windows.map.get(&WindowId::primary());

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = primary_window.map(|window| unsafe { instance.create_surface(window) });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: surface.as_ref(),
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty() | wgpu::Features::TEXTURE_BINDING_ARRAY,
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
        },
        None, // trace_path
    ))
    .unwrap();

    app.insert_resource(instance)
        .insert_resource(device)
        .insert_resource(queue);
}

fn test_create_more_windows(
    mut windows: ResMut<Windows>,
    mut create_window_event_writer: EventWriter<CreateWindow>,
) {
    create_window_event_writer.send(CreateWindow {
        id: windows.reserve_id(),
        desc: WindowDescriptor {},
    });
    create_window_event_writer.send(CreateWindow {
        id: windows.reserve_id(),
        desc: WindowDescriptor {},
    });
}

fn test_create_primary_camera(device: Res<wgpu::Device>, mut commands: Commands) {
    let camera_bundle = PerspectiveCameraBundle::new(&device);
    let primary_camera = commands.spawn().insert_bundle(camera_bundle).id();

    commands.insert_resource(PrimaryEntity::<Camera>::new(primary_camera));
}

fn test_create_render_entity(
    device: Res<wgpu::Device>,
    mut pipelines: ResMut<Store<RenderPipeline>>,
    mut bind_groups: ResMut<Store<wgpu::BindGroup>>,
    mut meshes: ResMut<Assets<Mesh<Vertex>>>,
    mut commands: Commands,
    primary_camera: Primary<Camera>,
    camera_query: Query<&Uniform<Camera>>,
) {
    let camera_uniform = camera_query.get(primary_camera.entity()).unwrap();

    let transform = Transform {
        rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(45.0)),
        ..Default::default()
    };
    let global_transform = GlobalTransform::from(transform);
    let model_uniform: Uniform<GlobalTransform> =
        Uniform::new(&device, global_transform.generate_uniform());

    let test_wgsl_binding_set = ((camera_uniform, &model_uniform),);
    let test_wgsl_pipeline = TestWgsl::pipeline(&device, test_wgsl_binding_set);
    let test_wgsl_binds = TestWgsl::bind_groups(&device, test_wgsl_binding_set);

    let cube_mesh = mesh::primitive::create_unit_cube();

    let refer_pipeline = store(&mut pipelines, test_wgsl_pipeline);
    let refer_binds = store_many(&mut bind_groups, test_wgsl_binds.into());
    let handle_cube_mesh = meshes.set(HandleId::random::<Mesh<Vertex>>(), cube_mesh);

    commands
        .spawn_bundle(RenderEntityBundle {
            pipeline: refer_pipeline,
            bind_groups: refer_binds,
            render_asset: handle_cube_mesh,
        })
        .insert(transform)
        .insert(global_transform);
}

fn reconfigure_camera_aspect(
    queue: &wgpu::Queue,
    camera: &mut Camera,
    camera_uniform: &mut Uniform<Camera>,
    perspective_projection: &mut PerspectiveProjection,
    new_size: &PhysicalSize<u32>,
) {
    perspective_projection.aspect = new_size.width as f32 / new_size.height as f32;
    camera.projection_matrix = perspective_projection.build_projection_matrix();
    camera.update_uniform(&mut camera_uniform.gpu_uniform);
    camera_uniform.sync_buffer(queue);
}

pub fn reconfigure_surface_system(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    mut surfaces: ResMut<Surfaces>,
    primary_camera: Primary<Camera>,
    mut camera_query: Query<(
        Entity,
        &mut Camera,
        &mut PerspectiveProjection,
        &mut Uniform<Camera>,
    )>,
    mut window_resize_events: EventReader<WindowResized>,
) {
    for WindowResized {
        id: window_id,
        new_size,
    } in window_resize_events.iter()
    {
        if new_size.width > 0 && new_size.height > 0 {
            println!(
                "Reconfiguring surface: {:?}, size: {:?}",
                window_id, new_size
            );
            let SurfaceKit { surface, config } = surfaces.0.get_mut(window_id).unwrap();
            config.width = new_size.width;
            config.height = new_size.height;
            surface.configure(&device, config);
            println!("Reconfigured");

            if window_id.is_primary() {
                let (_, mut camera, mut perspective_projection, mut camera_uniform) = camera_query
                    .iter_mut()
                    .find(|(entity, _, _, _)| entity.eq(&primary_camera.entity))
                    .unwrap();
                reconfigure_camera_aspect(
                    &queue,
                    &mut camera,
                    &mut camera_uniform,
                    &mut perspective_projection,
                    &new_size,
                )
            }
        }
    }
}

pub fn create_surface_system(
    device: Res<wgpu::Device>,
    instance: Res<wgpu::Instance>,
    mut surfaces: ResMut<Surfaces>,
    windows: Res<Windows>,
    mut create_window_events: EventReader<WindowCreated>,
) {
    let mut count = 0;
    for WindowCreated { id: window_id } in create_window_events.iter() {
        let window = windows.map.get(window_id).unwrap();
        let raw_window = &window.raw_window_handle;

        let size = window.physical_size;

        println!(
            "i: {}, Creating surface: {:?}, size: {:?}",
            count, window_id, size
        );

        let surface_kit =
            unsafe { create_surface(&instance, &device, &raw_window.get_handle(), size) };
        println!("Created");

        surfaces.0.insert(window_id.clone(), surface_kit);

        count += 1;
    }
}

unsafe fn create_surface<W>(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
    window: &W,
    size: PhysicalSize<u32>,
) -> SurfaceKit
where
    W: raw_window_handle::HasRawWindowHandle,
{
    let surface = instance.create_surface(window);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::engine_default(),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    SurfaceKit { surface, config }
}
