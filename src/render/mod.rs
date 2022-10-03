use bevy_app::Plugin;
use bevy_asset::AddAsset;
use bevy_ecs::{
    prelude::Component,
    system::{Query, Res},
};

use crate::{
    texture,
    util::{Refer, ReferMany, Store},
};

use self::{
    mesh::GpuMesh,
    resource::pipeline::RenderPipeline,
    resource::shader::{ShaderSource, ShaderSourceLoader, Shaders},
};

pub mod mesh;
pub mod mesh_bevy;
pub mod resource;

pub struct FlatRenderPlugin;
impl Plugin for FlatRenderPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Store<RenderPipeline>>()
            .init_resource::<Store<wgpu::BindGroup>>()
            .init_resource::<Shaders>()
            .add_asset_loader(ShaderSourceLoader)
            .add_asset::<ShaderSource>();
    }
}

// pub struct RenderAsset {
//     pipeline: wgpu::RenderPipeline,
//     bind_groups: Vec<wgpu::BindGroup>,
//     mesh: GpuMesh,
//     instance_data: wgpu::Buffer,
// }

#[derive(Component)]
pub struct InstanceData(wgpu::Buffer, u32);

pub struct DepthTexture(texture::Texture);

pub fn render_system(
    surface: Res<wgpu::Surface>,
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    depth_texture: Res<Option<DepthTexture>>,
    pipelines: Res<Store<RenderPipeline>>,
    bind_groups: Res<Store<wgpu::BindGroup>>,
    objects: Query<(
        &Refer<RenderPipeline>,
        &ReferMany<wgpu::BindGroup>,
        &GpuMesh,
        Option<&InstanceData>,
    )>,
) {
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

        for (pipeline, binds, mesh, instance) in objects.iter() {
            draw_mesh(
                &mut render_pass,
                pipelines.get(**pipeline).unwrap(),
                (*binds)
                    .iter()
                    .map(|i| bind_groups.get(*i).unwrap())
                    .collect::<Vec<_>>(),
                mesh,
                instance,
            );
        }
    } // drop(render_pass) <- mut borrow encoder <- mut borrow self

    queue.submit(std::iter::once(encoder.finish()));

    output.present();
}

fn draw_mesh<'a>(
    render_pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_groups: Vec<&'a wgpu::BindGroup>,
    mesh: &'a GpuMesh,
    instance: Option<&'a InstanceData>,
) {
    render_pass.set_pipeline(&pipeline.0);

    // TODO: binds are bound in the same order as they appear in RefMulti
    for (index, bind_group) in bind_groups.into_iter().enumerate() {
        render_pass.set_bind_group(index as u32, bind_group, &[]);
    }

    let mut instance_count = 1;
    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    if let Some(instance_data) = instance {
        render_pass.set_vertex_buffer(1, instance_data.0.slice(..));
        instance_count = instance_data.1;
    }

    match &mesh.assembly {
        mesh::GpuMeshAssembly::Indexed {
            index_buffer,
            index_count,
            index_format,
        } => {
            render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
            render_pass.draw_indexed(0..*index_count as u32, 0, 0..instance_count);
        }
        mesh::GpuMeshAssembly::NonIndexed { vertex_count } => {
            render_pass.draw(0..*vertex_count as u32, 0..instance_count);
        }
    }
}
