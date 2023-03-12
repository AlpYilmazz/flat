use bevy::{
    asset::{Asset, HandleId},
    prelude::{
        AddAsset, App, AssetEvent, Assets, CoreStage, Deref, DerefMut, EventReader,
        GlobalTransform, Handle, IntoSystemDescriptor, Plugin, Res, ResMut, Resource, StageLabel,
        SystemStage,
    },
    utils::HashMap,
    window::Windows,
};

use crate::util::NewTypePhantom;

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
    system::{render_system, RenderFunctions, RenderNode},
    texture::{Image, ImageLoader, ImageJustLoader},
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
            .init_asset_loader::<ShaderLoader>()
            .init_asset_loader::<ImageLoader>()
            .init_asset_loader::<ImageJustLoader>()
            // .init_asset_loader::<MeshLoader>()
            .add_asset::<Shader>()
            .add_render_asset::<Image>()
            .add_render_asset::<Mesh<Vertex>>()
            .add_component_uniform::<Color>()
            .add_component_uniform::<GlobalTransform>()
            .add_system_to_stage(RenderStage::Prepare, compile_shaders_into_pipelines);

        app.add_plugin(FlatCameraPlugin).add_plugin(FlatViewPlugin);

        create_wgpu_resources(app);
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
    let features = wgpu::Features::empty() | wgpu::Features::TEXTURE_BINDING_ARRAY;
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

pub trait AddRenderAsset {
    fn add_render_asset<T: RenderAsset>(&mut self) -> &mut Self;
}
impl AddRenderAsset for App {
    fn add_render_asset<T: RenderAsset>(&mut self) -> &mut Self {
        self.add_asset::<T>()
            .init_resource::<RenderAssets<T>>()
            .add_system_to_stage(RenderStage::Prepare, prepare_render_assets::<T>)
    }
}

pub trait RenderAsset: Asset {
    type PreparedAsset: Send + Sync + 'static;

    fn should_prepare(&self) -> bool {
        true
    }
    fn prepare(
        &self,
        render_device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Option<Self::PreparedAsset>;
}

#[derive(Resource, Deref, DerefMut)]
pub struct RenderAssets<T: RenderAsset>(pub HashMap<HandleId, T::PreparedAsset>);

impl<T: RenderAsset> Default for RenderAssets<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub type TryNextFrame<T> = NewTypePhantom<Vec<HandleId>, T>;

pub fn prepare_render_assets<T: RenderAsset>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    assets: Res<Assets<T>>,
    mut try_assets: ResMut<TryNextFrame<T>>, // NOTE: Infinite growth
    mut render_assets: ResMut<RenderAssets<T>>,
    mut asset_events: EventReader<AssetEvent<T>>,
) {
    let try_assets_take = std::mem::replace(&mut try_assets.0, Vec::new());
    for handle_id in try_assets_take {
        if let Some(asset) = assets.get(&Handle::weak(handle_id)) {
            match asset.prepare(&render_device, &render_queue) {
                Some(render_asset) => {
                    render_assets.insert(handle_id, render_asset);
                }
                None => {
                    if asset.should_prepare() {
                        try_assets.push(handle_id);
                    }
                }
            }
        }
    }

    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let handle_id = handle.id();
                if let Some(asset) = assets.get(handle) {
                    match asset.prepare(&render_device, &render_queue) {
                        Some(render_asset) => {
                            render_assets.insert(handle_id, render_asset);
                        }
                        None => {
                            if asset.should_prepare() {
                                try_assets.push(handle_id);
                            }
                        }
                    }
                }
            }
            AssetEvent::Removed { handle } => {
                render_assets.remove(&handle.id());
            }
        }
    }
}
