use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy_app::{App, Plugin};
use bevy_asset::{AddAsset, Asset, AssetEvent, Assets, Handle, HandleId};
use bevy_ecs::{
    prelude::{Component, Entity, EventReader, Bundle},
    query::{Added, Changed, With},
    system::{Commands, Query, RemovedComponents, Res, ResMut, SystemParam, lifetimeless::{SQuery, Read}, SystemParamItem, StaticSystemParam},
};

use crate::{
    transform::GlobalTransform,
    util::{AssetStore, Location, LocationBound, Refer, Sink, Store},
};

use super::{
    camera::{ExtractedCameras, Visible},
    mesh::GpuMesh,
    resource::{bind::{InnerBindingSet, BindingSet}, pipeline::RenderPipeline},
    transform::ExtractedTransform,
    DepthTextures, InstanceData, RenderCamera, RenderStage, SurfaceKit, Surfaces,
};

pub trait AddAssetExtract {
    fn add_asset_extract<T: RenderAsset>(&mut self) -> &mut Self;
}

impl AddAssetExtract for App {
    fn add_asset_extract<T: RenderAsset>(&mut self) -> &mut Self {
        self.add_asset::<T>()
            .init_resource::<ExtractedAssets<T::ExtractedAsset>>()
            .add_system_to_stage(RenderStage::Extract, extract_render_assets::<T>)
    }
}

pub trait AddRenderSystem {
    fn add_render_system<T: Draw>(&mut self) -> &mut Self;
}

impl AddRenderSystem for App {
    fn add_render_system<T: Draw>(&mut self) -> &mut Self {
        self.add_system_to_stage(RenderStage::Render, render_system::<T>)
    }
}

pub struct RenderPlugin<T: RenderAsset<ExtractedAsset = GpuMesh>>(PhantomData<fn() -> T>);
impl<T: RenderAsset<ExtractedAsset = GpuMesh>> Default for RenderPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T: RenderAsset<ExtractedAsset = GpuMesh>> Plugin for RenderPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_asset_extract::<T>()
            .add_system_to_stage(RenderStage::Prepare, insert_render_handle::<T>);
    }
}

#[derive(Component)]
pub struct RenderHandle<T: Send + Sync + 'static>(HandleId, PhantomData<fn() -> T>);
impl<T: Send + Sync + 'static> RenderHandle<T> {
    pub fn new(id: HandleId) -> Self {
        Self(id, PhantomData)
    }

    pub fn id(&self) -> HandleId {
        self.0
    }
}

pub trait Draw: Send + Sync + 'static {
    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_data: Option<&'a InstanceData>,
    );
}

pub trait RenderAsset: Asset {
    type ExtractedAsset: Send + Sync + 'static;

    fn extract(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Self::ExtractedAsset;
}

pub struct ExtractedAssets<T>(AssetStore<T>);
impl<T> Default for ExtractedAssets<T> {
    fn default() -> Self {
        Self(AssetStore::default())
    }
}
impl<T> Deref for ExtractedAssets<T> {
    type Target = AssetStore<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for ExtractedAssets<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn extract_render_assets<T: RenderAsset>(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    mut extracted_assets: ResMut<ExtractedAssets<T::ExtractedAsset>>,
    assets: Res<Assets<T>>,
    mut asset_events: EventReader<AssetEvent<T>>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let handle_id = handle.into();
                let extracted_asset = assets.get(&handle).unwrap().extract(&device, &queue);
                extracted_assets.insert(handle_id, extracted_asset);
            }
            AssetEvent::Removed { handle } => {
                let handle_id = handle.into();
                extracted_assets.remove(&handle_id);
            }
        }
    }
}

pub fn insert_render_handle<T: RenderAsset>(
    mut commands: Commands,
    render_entities: Query<(Entity, &Handle<T>), Changed<Handle<T>>>,
    removed_handles: RemovedComponents<Handle<T>>,
) {
    for (entity, handle) in render_entities.iter() {
        commands
            .entity(entity)
            .insert(RenderHandle::<T::ExtractedAsset>::new(handle.id));
        removed_handles.iter().for_each(|entity| {
            commands
                .entity(entity)
                .remove::<RenderHandle<T::ExtractedAsset>>()
                .sink()
        });
    }
}

pub fn render_system<T: Draw>(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    // mut command_buffers: ResMut<CommandBuffers>,
    // surface_textures: ResMut<SurfaceTextures>,
    surfaces: Res<Surfaces>,
    depth_textures: Res<DepthTextures>,
    pipelines: Res<Store<RenderPipeline>>,
    bind_groups: Res<Store<wgpu::BindGroup>>,
    cameras: Res<ExtractedCameras>,
    extracted_assets: Res<ExtractedAssets<T>>,
    render_entities: Query<
        (
            Entity,
            &Refer<RenderPipeline>,
            &RenderCamera,
            &ExtractedTransform,
            &RenderHandle<T>,
            Option<&InstanceData>,
        ),
        (With<GlobalTransform>, With<Visible>),
    >,
) {
    for (window_id, SurfaceKit { surface, .. }) in surfaces.iter() {
        let depth_texture = depth_textures.get(window_id);

        let output = match surface.get_current_texture() {
            Ok(output) => output,
            // Reconfigure the surface if lost
            // Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
            // The system is out of memory, we should probably quit
            // Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: depth_texture.map(|dt| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &dt.0.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }
                }),
            });

            for (
                entity,
                refer_pipeline,
                camera,
                extracted_transform,
                draw_asset_handle,
                instance_data,
            ) in render_entities.iter()
            {
                let extracted_camera = cameras.get(&camera.get()).unwrap();
                // let extracted_camera = match extracted_camera {
                //     Some(c) => c,
                //     None => continue,
                // };

                if extracted_camera.render_window != *window_id {
                    continue;
                }

                let pipeline = pipelines.get(**refer_pipeline).unwrap();
                // let pipeline = match pipeline {
                //     Some(p) => p,
                //     None => continue,
                // };

                let bind_groups = [
                    bind_groups.get(*extracted_camera.bind_refer).unwrap(),
                    bind_groups.get(*extracted_transform.bind_refer).unwrap(),
                ];

                if let Some(render_asset_entity) =
                    extracted_assets.get(&draw_asset_handle.id().into())
                {
                    draw_gpu_mesh(
                        &mut render_pass,
                        pipeline,
                        &bind_groups,
                        render_asset_entity,
                        instance_data,
                    );
                }
            }
        } // drop(render_pass) <- mut borrow encoder

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

fn draw_gpu_mesh<'a, T: Draw>(
    render_pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_groups: &[&'a wgpu::BindGroup],
    render_entity: &'a T,
    instance_data: Option<&'a InstanceData>,
) {
    render_pass.set_pipeline(&pipeline.inner);

    // TODO: binds are bound in the same order as they appear in RefMany
    for (index, bind_group) in bind_groups.iter().enumerate() {
        render_pass.set_bind_group(index as u32, bind_group, &[]);
    }

    render_entity.draw(render_pass, instance_data);
}

#[derive(Component)]
pub struct BindRegistry {
    pub map: HashMap<Location, wgpu::BindGroup>,
    pub cache: HashMap<TypeId, Location>,
}

pub fn extract_bind_system<T>(
    device: Res<wgpu::Device>,
    mut query: Query<(&T, &<T as InnerBindingSet>::InnerDesc, &mut BindRegistry), Added<T>>,
    // removed: RemovedComponents<T>,
) where
    T: Component + InnerBindingSet + LocationBound,
    <T as InnerBindingSet>::InnerDesc: Component,
{
    for (bind, desc, mut registry) in query.iter_mut() {
        let bind_group = bind.extract_bind_group(&device, desc);
        registry.map.insert(bind.get_location(), bind_group);
    }
}

pub fn into_bind_system<T>(
    device: Res<wgpu::Device>,
    mut query: Query<(&T, &<T as BindingSet>::SetDesc, &mut BindRegistry), Added<T>>,
    // removed: RemovedComponents<T>,
) where
    T: Component + BindingSet + LocationBound,
    <T as BindingSet>::SetDesc: Component,
{
    for (bind, desc, mut registry) in query.iter_mut() {
        let bind_group = bind.into_bind_group(&device, desc);
        registry.map.insert(bind.get_location(), bind_group);
    }
}

pub trait BindStruct {
    type Fetch: SystemParam;

    fn set_binds(render_pass: &mut wgpu::RenderPass, fetch: &mut SystemParamItem<Self::Fetch>);
}

pub struct MVPBindStruct {
    camera: RenderCamera,
    model: GlobalTransform,
}
impl BindStruct for MVPBindStruct {
    type Fetch = (
        SQuery<(Read<RenderCamera>, Read<GlobalTransform>)>,
    );

    fn set_binds(render_pass: &mut wgpu::RenderPass, (query,): &mut SystemParamItem<Self::Fetch>) {
        todo!()
    }
}

fn get_render_pass() -> &'static mut wgpu::RenderPass<'static> {
    todo!()
}
fn bind_struct_system<T: BindStruct>(
    fetch: StaticSystemParam<<T as BindStruct>::Fetch>,
) {
    let mut fetch = fetch.into_inner();
    T::set_binds(get_render_pass(), &mut fetch);
}

fn test() {
    let mut app = App::new();
    app.add_system(bind_struct_system::<MVPBindStruct>);
}
