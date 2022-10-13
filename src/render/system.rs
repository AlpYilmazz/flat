use std::collections::HashMap;

use bevy_app::App;
use bevy_asset::{AddAsset, Asset, AssetEvent, Assets, Handle};
use bevy_ecs::{
    prelude::{Entity, EventReader},
    query::{Added, Changed, Or, With},
    system::{Query, RemovedComponents, Res, ResMut},
};

use crate::{
    render::resource::{bind::BindingSet, uniform::UniformDesc},
    transform::GlobalTransform,
    util::{store, AssetStore, Refer, Store, Sink},
};

use super::{
    camera::{ExtractedCameras, Visible},
    resource::{
        pipeline::RenderPipeline,
        uniform::{HandleGpuUniform, Uniform},
    },
    DepthTexture, InstanceData, RenderCamera, RenderStage, SurfaceKit, Surfaces,
};

pub trait AddRenderSystem {
    fn add_render_system<T: RenderAsset>(&mut self) -> &mut Self;
}

impl AddRenderSystem for App {
    fn add_render_system<T: RenderAsset>(&mut self) -> &mut Self {
        self.add_asset::<T>()
            .init_resource::<RenderAssets<T::GpuEntity>>()
            .init_resource::<ExtractedTransforms>()
            .add_system_to_stage(RenderStage::Extract, extract_global_transforms::<T>)
            .add_system_to_stage(RenderStage::Extract, extract_render_assets::<T>)
            .add_system_to_stage(RenderStage::Render, render_system::<T>)
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

pub fn extract_render_assets<T: RenderAsset>(
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

pub struct ExtractedTransform {
    pub uniform: Uniform<GlobalTransform>,
    pub bind_refer: Refer<wgpu::BindGroup>,
}
pub type ExtractedTransforms = HashMap<Entity, ExtractedTransform>;

pub fn extract_global_transforms<T: RenderAsset>(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    query: Query<
        (
            Entity,
            &GlobalTransform,
            Added<GlobalTransform>,
            Changed<GlobalTransform>,
            &Handle<T>,
        ),
        Or<(Added<GlobalTransform>, Changed<GlobalTransform>)>,
    >,
    removed: RemovedComponents<GlobalTransform>,
    mut extracted_transforms: ResMut<ExtractedTransforms>,
    mut bind_groups: ResMut<Store<wgpu::BindGroup>>,
) {
    for (entity, gtransform, added, changed, _handle) in query.iter() {
        if added {
            assert!(!extracted_transforms.contains_key(&entity));

            let uniform = Uniform::new(&device, gtransform.generate_uniform());
            let bind_group = uniform.as_ref().into_bind_group(&device, &UniformDesc::default());
            let bind_refer = store(&mut bind_groups, bind_group);
            extracted_transforms.insert(
                entity,
                ExtractedTransform {
                    uniform,
                    bind_refer,
                },
            );
        } else if changed {
            let ExtractedTransform { uniform, .. } = extracted_transforms.get_mut(&entity).unwrap();
            gtransform.update_uniform(&mut uniform.gpu_uniform);
            uniform.sync_buffer(&queue);
        };
    }

    removed
        .iter()
        .for_each(|entity| extracted_transforms.remove(&entity).sink());
}

pub fn render_system<T: RenderAsset>(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    surfaces: Res<Surfaces>,
    depth_texture: Option<Res<DepthTexture>>,
    pipelines: Res<Store<RenderPipeline>>,
    bind_groups: Res<Store<wgpu::BindGroup>>,
    cameras: Res<ExtractedCameras>,
    models: Res<ExtractedTransforms>,
    render_assets: Res<RenderAssets<T::GpuEntity>>,
    render_entities: Query<
        (
            Entity,
            &Refer<RenderPipeline>,
            &RenderCamera,
            &Handle<T>,
            Option<&InstanceData>,
        ),
        (With<GlobalTransform>, With<Visible>),
    >,
) {
    for (window_id, SurfaceKit { surface, .. }) in surfaces.0.iter() {
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
            });

            for (entity, refer_pipeline, camera, render_asset_handle, instance_data) in
                render_entities.iter()
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
                    bind_groups
                        .get(*models.get(&entity).unwrap().bind_refer)
                        .unwrap(),
                ];

                draw_entity(
                    &mut render_pass,
                    pipeline,
                    &bind_groups,
                    render_assets.0.get(&render_asset_handle.into()).unwrap(),
                    instance_data,
                );
            }
        } // drop(render_pass) <- mut borrow encoder

        queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }
}

fn draw_entity<'a, T: RenderEntity>(
    render_pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_groups: &[&'a wgpu::BindGroup],
    render_entity: &'a T,
    instance_data: Option<&'a InstanceData>,
) {
    render_pass.set_pipeline(&pipeline.0);

    // TODO: binds are bound in the same order as they appear in RefMany
    for (index, bind_group) in bind_groups.iter().enumerate() {
        render_pass.set_bind_group(index as u32, bind_group, &[]);
    }

    render_entity.set_buffers(render_pass, instance_data);
}
