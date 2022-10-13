use std::collections::HashMap;

use bevy_app::{App, CoreStage, Plugin, StartupStage};
use bevy_asset::{AddAsset, Assets, Handle, HandleId};
use bevy_ecs::{
    prelude::{Bundle, Component, Entity, EventReader, EventWriter},
    schedule::{ParallelSystemDescriptorCoercion, StageLabel, SystemLabel, SystemStage},
    system::{Commands, Query, Res, ResMut},
};
use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use winit::dpi::PhysicalSize;

use crate::{
    shaders::{ShaderInstance, TestWgsl},
    texture,
    transform::{GlobalTransform, Transform},
    util::{store, AssetStore, EngineDefault, Primary, PrimaryEntity, Refer, Store},
    window::{
        events::{CreateWindow, WindowCreated, WindowResized},
        WindowDescriptor, WindowId, Windows, WinitWindows,
    },
};

use self::{
    camera::{Camera, FlatCameraPlugin, PerspectiveCameraBundle, PerspectiveProjection, Visible},
    mesh::Mesh,
    resource::shader::{ShaderSource, ShaderSourceLoader, Shaders},
    resource::{buffer::Vertex, pipeline::RenderPipeline},
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

#[derive(SystemLabel)]
pub struct SurfaceReconfigure;

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
        .init_resource::<AssetStore<Refer<RenderPipeline>>>()
        .init_resource::<Store<wgpu::BindGroup>>()
        .init_resource::<Shaders>()
        .add_asset_loader(ShaderSourceLoader)
        .add_asset::<ShaderSource>()
        .add_system_to_stage(CoreStage::PreUpdate, create_surface_system)
        .add_system_to_stage(
            RenderStage::Prepare,
            reconfigure_surface_system.label(SurfaceReconfigure),
        )
        .add_render_system::<Mesh<Vertex>>();

        create_wgpu_resources(app);

        app.add_startup_system_to_stage(StartupStage::PreStartup, test_create_pipeline_test_wgsl)
            .add_startup_system_to_stage(StartupStage::PreStartup, test_create_primary_camera)
            .add_startup_system(test_create_render_entity);
        // .add_startup_system(test_create_more_windows);

        app.add_plugin(FlatCameraPlugin);
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
    pub render_camera: RenderCamera,
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub render_asset: Handle<T>,
}

#[derive(Debug, Clone, Component)]
pub struct RenderCamera(pub Entity);
impl RenderCamera {
    pub fn get(&self) -> Entity {
        self.0
    }
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

fn test_create_pipeline_test_wgsl(
    device: Res<wgpu::Device>,
    mut render_pipelines: ResMut<Store<RenderPipeline>>,
    mut refer_pipelines: ResMut<AssetStore<Refer<RenderPipeline>>>,
) {
    let handle_id = HandleId::from("test.wgsl");
    let refer = store(&mut render_pipelines, TestWgsl::pipeline(&device));
    refer_pipelines.insert(handle_id, refer);
}

fn test_create_primary_camera(mut commands: Commands) {
    let camera_bundle = PerspectiveCameraBundle::new(WindowId::primary()); // Primary Window
    let primary_camera = commands.spawn().insert_bundle(camera_bundle).id();

    commands.insert_resource(PrimaryEntity::<Camera>::new(primary_camera));
}

#[derive(Component)]
pub struct Player;

fn test_create_render_entity(
    mut commands: Commands,
    primary_camera: Primary<Camera>,
    mut meshes: ResMut<Assets<Mesh<Vertex>>>,
    pipelines: Res<AssetStore<Refer<RenderPipeline>>>,
) {
    let pipeline = pipelines.get(&HandleId::from("test.wgsl")).unwrap().clone();
    let render_camera = RenderCamera(primary_camera.get());

    let transform = Transform {
        rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(45.0)),
        ..Default::default()
    };
    let global_transform = GlobalTransform::from(transform);

    let cube_mesh = mesh::primitive::create_unit_cube();
    let handle_cube_mesh = meshes.set(HandleId::random::<Mesh<Vertex>>(), cube_mesh);

    commands
        .spawn_bundle(RenderEntityBundle {
            pipeline,
            render_camera,
            global_transform,
            transform,
            render_asset: handle_cube_mesh,
        })
        .insert(Visible)
        .insert(Player);
}

pub fn reconfigure_surface_system(
    device: Res<wgpu::Device>,
    mut surfaces: ResMut<Surfaces>,
    mut camera_query: Query<(&Camera, &mut PerspectiveProjection)>,
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

            for (camera, mut perspective_projection) in camera_query.iter_mut() {
                if camera.render_window.eq(window_id) {
                    perspective_projection.aspect = new_size.width as f32 / new_size.height as f32;
                }
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
