use std::collections::HashMap;

use bevy_app::{App, CoreStage, Plugin};
use bevy_asset::{AddAsset, Handle};
use bevy_ecs::{
    prelude::{Bundle, Component, Entity, EventReader},
    schedule::{ParallelSystemDescriptorCoercion, StageLabel, SystemLabel, SystemStage},
    system::{Res, ResMut},
};
use winit::dpi::PhysicalSize;

use crate::{
    transform::{GlobalTransform, Transform},
    util::{AssetStore, EngineDefault, Refer, Store},
    window::{
        events::{WindowClosed, WindowCreated, WindowResized},
        WindowId, Windows, WinitWindows,
    },
};

use self::{
    camera::RenderCameraPlugin,
    mesh::{extend::Quad, GpuMesh, Mesh},
    resource::shader::{ShaderSource, ShaderSourceLoader},
    resource::{buffer::Vertex, pipeline::RenderPipeline},
    system::{AddRenderSystem, RenderAsset, RenderPlugin},
    texture::GpuTexture,
    transform::RenderTransformPlugin,
};

pub mod camera;
pub mod command;
pub mod mesh;
pub mod mesh_bevy;
pub mod resource;
pub mod system;
pub mod texture;
pub mod transform;

#[derive(StageLabel)]
pub enum RenderStage {
    Prepare,
    Extract,
    Render,
    Present,
}

#[derive(SystemLabel)]
pub struct SurfaceLifecycle;
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
            SystemStage::single_threaded(),
        )
        .add_stage_after(
            RenderStage::Extract,
            RenderStage::Present,
            SystemStage::parallel(),
        )
        .init_resource::<DepthTextures>()
        .init_resource::<Surfaces>()
        .init_resource::<Store<RenderPipeline>>()
        .init_resource::<AssetStore<Refer<RenderPipeline>>>()
        .init_resource::<Store<wgpu::BindGroup>>()
        .add_asset_loader(ShaderSourceLoader)
        .add_asset::<ShaderSource>()
        .add_system_to_stage(
            RenderStage::Prepare,
            create_surface_system.label(SurfaceLifecycle),
        )
        .add_system_to_stage(
            RenderStage::Prepare,
            destroy_surface_system.label(SurfaceLifecycle),
        )
        .add_system_to_stage(
            RenderStage::Prepare,
            reconfigure_surface_system
                .label(SurfaceReconfigure)
                .after(SurfaceLifecycle),
        );

        create_wgpu_resources(app);

        // app.add_asset_extract::<Image>();

        app.add_render_system::<GpuMesh>()
            .add_plugin(RenderPlugin::<Mesh<Vertex>>::default())
            .add_plugin(RenderPlugin::<Quad>::default())
            .add_plugin(RenderCameraPlugin)
            .add_plugin(RenderTransformPlugin);
    }
}

#[derive(Component)]
pub struct InstanceData(wgpu::Buffer, u32);

pub struct DepthTexture(texture::GpuTexture);
pub type DepthTextures = HashMap<WindowId, DepthTexture>;

pub struct SurfaceKit {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}
pub type Surfaces = HashMap<WindowId, SurfaceKit>;

#[derive(Bundle)]
pub struct RenderEntityBundle<T: RenderAsset> {
    pub pipeline: Refer<RenderPipeline>,
    pub render_camera: RenderCamera,
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub render_asset: Handle<T>,
}

#[derive(Debug, Clone, Copy, Component)]
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

pub type CommandBuffers = Vec<wgpu::CommandBuffer>;
pub type SurfaceTextures = Vec<(WindowId, wgpu::SurfaceTexture, wgpu::TextureView)>;

pub fn reconfigure_surface_system(
    device: Res<wgpu::Device>,
    mut surfaces: ResMut<Surfaces>,
    mut depth_textures: ResMut<DepthTextures>,
    mut window_resize_events: EventReader<WindowResized>,
) {
    for WindowResized {
        id: window_id,
        new_size,
    } in window_resize_events.iter()
    {
        if new_size.width > 0 && new_size.height > 0 {
            let SurfaceKit { surface, config } = surfaces.get_mut(window_id).unwrap();
            config.width = new_size.width;
            config.height = new_size.height;
            surface.configure(&device, config);

            let depth_texture = depth_textures.get_mut(window_id);
            depth_texture.map(|dt| {
                *dt = DepthTexture(GpuTexture::create_depth_texture(&device, config, None));
            });
        }
    }
}

pub fn create_surface_system(
    device: Res<wgpu::Device>,
    instance: Res<wgpu::Instance>,
    mut surfaces: ResMut<Surfaces>,
    mut depth_textures: ResMut<DepthTextures>,
    windows: Res<Windows>,
    mut window_created_events: EventReader<WindowCreated>,
) {
    for WindowCreated { id: window_id } in window_created_events.iter() {
        let window = windows.map.get(window_id).unwrap();
        let raw_window = &window.raw_window_handle;

        let size = window.physical_size;

        let surface_kit =
            unsafe { create_surface(&instance, &device, &raw_window.get_handle(), size) };

        depth_textures.insert(
            window_id.clone(),
            DepthTexture(GpuTexture::create_depth_texture(
                &device,
                &surface_kit.config,
                None,
            )),
        );

        surfaces.insert(window_id.clone(), surface_kit);
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

pub fn destroy_surface_system(
    mut surfaces: ResMut<Surfaces>,
    mut depth_textures: ResMut<DepthTextures>,
    mut window_closed_events: EventReader<WindowClosed>,
) {
    for WindowClosed { id: window_id } in window_closed_events.iter() {
        surfaces.remove(window_id);
        depth_textures.remove(window_id);
    }
}
