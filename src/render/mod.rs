use bevy::{
    asset::{Asset, HandleId},
    prelude::{
        AddAsset, App, AssetEvent, Assets, CoreStage, Deref, DerefMut, EventReader,
        GlobalTransform, IntoSystemDescriptor, Plugin, Res, ResMut, Resource, StageLabel,
        SystemStage,
    },
    utils::HashMap,
    window::Windows,
};

use self::{
    camera::FlatCameraPlugin,
    color::Color,
    mesh::Mesh,
    resource::{
        buffer::Vertex,
        component_uniform::AddComponentUniform,
        pipeline::{compile_shaders_into_pipelines, PipelineCache},
        renderer::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue},
        shader::{Shader, ShaderLoader},
    },
    system::{render_system, AddRenderFunction, RenderFunctions, RenderNode},
    texture::{Image, ImageLoader},
    view::window::FlatViewPlugin,
};

pub mod camera;
pub mod color;
pub mod mesh;
pub mod resource;
pub mod system;
pub mod texture;
pub mod view;

#[derive(StageLabel)]
pub enum RenderStage {
    Prepare, // Prepare Resources and Entities for the rendering context: Image -> GpuTexture
    Create,  // Create resources directly for rendering: GpuTexture -> BindGroup
    Render,  // Render
    Cleanup, // Cleanup
}

pub struct FlatRenderPlugin;
impl Plugin for FlatRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_stage_after(
            CoreStage::PostUpdate,
            RenderStage::Prepare,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RenderStage::Prepare,
            RenderStage::Create,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RenderStage::Create,
            RenderStage::Render,
            SystemStage::parallel().with_system(render_system.at_end()),
        )
        .add_stage_after(
            RenderStage::Render,
            RenderStage::Cleanup,
            SystemStage::parallel(),
        );

        app.init_resource::<RenderFunctions>()
            .init_resource::<RenderNode>()
            .init_resource::<PipelineCache>()
            .add_asset::<Shader>()
            .add_asset::<Image>()
            .add_asset::<Mesh<Vertex>>()
            .init_asset_loader::<ShaderLoader>()
            .init_asset_loader::<ImageLoader>()
            // .init_asset_loader::<MeshLoader>()
            .add_component_uniform::<Color>()
            .add_component_uniform::<GlobalTransform>()
            .add_system_to_stage(RenderStage::Prepare, compile_shaders_into_pipelines)
            .add_system_to_stage(RenderStage::Prepare, prepare_render_assets::<Image>)
            .add_system_to_stage(RenderStage::Prepare, prepare_render_assets::<Mesh<Vertex>>);

        app.add_plugin(FlatCameraPlugin).add_plugin(FlatViewPlugin);

        create_wgpu_resources(app);

        app.complete_render_function_init();
    }
}

///
/// Creates wgpu Instance, Device and Queue as World Resources.
///
/// Creates wpgu Surface for initial primary window.
///
pub fn create_wgpu_resources(app: &mut App) {
    let backends = wgpu::Backends::all();
    let power_preference = wgpu::PowerPreference::HighPerformance;
    let features = wgpu::Features::empty();
    let limits = if cfg!(target_arch = "wasm32") {
        wgpu::Limits::downlevel_webgl2_defaults()
    } else {
        wgpu::Limits::default()
    };

    let windows = app.world.resource::<Windows>();
    let instance = wgpu::Instance::new(backends);

    let surface = windows
        .get_primary()
        .and_then(|window| window.raw_handle())
        .map(|wrapper| unsafe {
            let handle = wrapper.get_handle();
            instance.create_surface(&handle)
        });

    if let None = surface {
        println!("PRIMARY WINDOW NONE")
    } else {
        println!("Primary Window: {}", windows.primary().title());
    }

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let adapter =
        futures_lite::future::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: surface.as_ref(),
            ..Default::default()
        }))
        .unwrap();

    let (device, queue) = futures_lite::future::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features,
            limits,
        },
        None, // trace_path
    ))
    .unwrap();

    app.insert_resource(RenderInstance(instance))
        .insert_resource(RenderAdapter(adapter))
        .insert_resource(RenderQueue(queue))
        .insert_resource(RenderDevice(device));
}

pub trait RenderAsset: Asset {
    type PreparedAsset: Send + Sync + 'static;

    fn prepare(&self, render_device: &RenderDevice, queue: &RenderQueue) -> Self::PreparedAsset;
}

#[derive(Resource, Deref, DerefMut)]
pub struct RenderAssets<T: RenderAsset>(pub HashMap<HandleId, T::PreparedAsset>);

pub fn prepare_render_assets<T: RenderAsset>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    assets: Res<Assets<T>>,
    mut render_assets: ResMut<RenderAssets<T>>,
    mut asset_events: EventReader<AssetEvent<T>>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let handle_id = handle.id();
                if let Some(asset) = assets.get(handle) {
                    render_assets.insert(handle_id, asset.prepare(&render_device, &render_queue));
                }
            }
            AssetEvent::Removed { handle } => {
                render_assets.remove(&handle.id());
            }
        }
    }
}
