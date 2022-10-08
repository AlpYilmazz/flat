use bevy_app::App;
use bevy_asset::{AddAsset, Asset, AssetEvent, Assets, Handle};
use bevy_ecs::{
    prelude::EventReader,
    system::{Query, Res, ResMut},
};

use crate::{
    util::{AssetStore, Refer, ReferMany, Store},
    window::WindowId,
};

use super::{
    resource::pipeline::RenderPipeline, DepthTexture, InstanceData, RenderStage, Surfaces,
};

pub trait AddRenderSystem {
    fn add_render_system<T: RenderAsset>(&mut self);
}

impl AddRenderSystem for App {
    fn add_render_system<T: RenderAsset>(&mut self) {
        self.add_asset::<T>()
            .init_resource::<RenderAssets<T::GpuEntity>>()
            .add_system_to_stage(RenderStage::Extract, extract_render_asset::<T>)
            .add_system_to_stage(RenderStage::Render, render_system::<T>);
    }
}

pub trait RenderEntity: Send + Sync + 'static {
    fn set_buffers<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instance_data: Option<&'a InstanceData>,
    );
}

pub trait RenderAsset: Asset {
    type GpuEntity: RenderEntity;

    fn extract(&self, device: &wgpu::Device) -> Self::GpuEntity;
}

pub struct RenderAssets<T: RenderEntity>(pub AssetStore<T>);

impl<T: RenderEntity> Default for RenderAssets<T> {
    fn default() -> Self {
        Self(AssetStore::default())
    }
}

pub fn extract_render_asset<T: RenderAsset>(
    device: Res<wgpu::Device>,
    mut render_assets: ResMut<RenderAssets<T::GpuEntity>>,
    assets: Res<Assets<T>>,
    mut asset_events: EventReader<AssetEvent<T>>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let handle_id = handle.into();
                let extracted_asset = assets.get(&handle).unwrap().extract(&device);
                render_assets.0.insert(handle_id, extracted_asset);
            }
            AssetEvent::Removed { handle } => {
                let handle_id = handle.into();
                render_assets.0.remove(&handle_id);
            }
        }
    }
}

pub fn render_system<T: RenderAsset>(
    surfaces: Res<Surfaces>,
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    depth_texture: Option<Res<DepthTexture>>,
    pipelines: Res<Store<RenderPipeline>>,
    bind_groups: Res<Store<wgpu::BindGroup>>,
    render_assets: Res<RenderAssets<T::GpuEntity>>,
    render_entities: Query<(
        &Refer<RenderPipeline>,
        &ReferMany<wgpu::BindGroup>,
        &Handle<T>,
        Option<&InstanceData>,
    )>,
) {
    // TODO: only renders onto the primary window
    let surface = &surfaces.0.get(&WindowId::primary()).unwrap().surface;

    let output = surface.get_current_texture().unwrap();
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
            depth_stencil_attachment: depth_texture.as_ref().as_ref().map(|dt| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: &dt.0.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }
            }),
            // depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            //     view: &(
            //         depth_texture
            //         .as_ref()
            //         .as_ref()
            //         .unwrap()
            //         .0
            //         .view
            //     ),
            //     depth_ops: Some(wgpu::Operations {
            //         load: wgpu::LoadOp::Clear(1.0),
            //         store: true,
            //     }),
            //     stencil_ops: None,
            // }),
        });

        for (pipeline, binds, render_asset_handle, instance) in render_entities.iter() {
            draw_entity(
                &mut render_pass,
                pipelines.get(**pipeline).unwrap(),
                (*binds)
                    .iter()
                    .map(|i| bind_groups.get(*i).unwrap())
                    .collect::<Vec<_>>(),
                render_assets.0.get(&render_asset_handle.into()).unwrap(),
                instance,
            );
        }
    } // drop(render_pass) <- mut borrow encoder

    queue.submit(std::iter::once(encoder.finish()));

    output.present();
}

fn draw_entity<'a, T: RenderEntity>(
    render_pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_groups: Vec<&'a wgpu::BindGroup>,
    render_entity: &'a T,
    instance_data: Option<&'a InstanceData>,
) {
    render_pass.set_pipeline(&pipeline.0);

    // TODO: binds are bound in the same order as they appear in RefMulti
    for (index, bind_group) in bind_groups.into_iter().enumerate() {
        render_pass.set_bind_group(index as u32, bind_group, &[]);
    }

    render_entity.set_buffers(render_pass, instance_data);
}
