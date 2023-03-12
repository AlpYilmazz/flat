use bevy::{prelude::{HandleUntyped, Plugin, Assets, Entity, World, Handle}, reflect::TypeUuid, asset::load_internal_asset};

use crate::{render::{resource::{shader::Shader, buffer::Vertex3DTex, pipeline::PipelineCache, uniform::DynamicUniformId, component_uniform::ModelUniform}, mesh::{Mesh, GpuMeshAssembly}, system::{RenderResult, AddRenderFunction}, RenderStage, RenderAssets, camera::component::CameraUniforms}, shapes::skybox::create_skybox, mesh3d::bind::{Mesh3DPipeline, Mesh3DBindGroups, create_mesh3d_bind_groups}, sprite::bind::TextureBindGroups};

use self::bundle::Textures;

pub mod bind;
pub mod bundle;

const MESH3D_3TEX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 15678909876445673);

pub const BASE_CUBE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 15678909876445674);

pub struct FlatSpritePlugin;
impl Plugin for FlatSpritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, MESH3D_3TEX_SHADER_HANDLE, "mesh_texarr.wgsl", Shader::from_wgsl);

        {
            let mut meshes = app
                .world
                .get_resource_mut::<Assets<Mesh<Vertex3DTex>>>()
                .unwrap();
            meshes.set_untracked(BASE_CUBE_HANDLE, create_skybox());
        }

        app.init_resource::<Mesh3DPipeline<6>>()
            .init_resource::<Mesh3DBindGroups>()
            .add_render_function(MESH3D_RENDER_FUNCTION, render_mesh::<6>)
            .add_system_to_stage(RenderStage::Create, create_mesh3d_bind_groups::<6>);
    }
}

const MESH3D_RENDER_FUNCTION: usize = 2;
fn render_mesh<'w, const N: usize>(
    camera: Entity,
    object: Entity,
    world: &'w World,
    render_pass: &mut wgpu::RenderPass<'w>,
) -> RenderResult {
    // -- Set Pipeline --
    let mesh3d_pipeline = world.get_resource::<Mesh3DPipeline<N>>().unwrap();
    let pipeline_cache = world.get_resource::<PipelineCache>().unwrap();
    let Some(render_pipeline) = pipeline_cache.get(&mesh3d_pipeline.pipeline_id) else {
        return RenderResult::Failure;
    };
    render_pass.set_pipeline(render_pipeline);
    // -- -- -- -------- -- -- --

    // -- Get Mesh --
    let Some(mesh_handle) = world.get::<Handle<Mesh<Vertex3DTex>>>(object) else {
        return RenderResult::Failure;
    };
    let gpu_meshes = world.get_resource::<RenderAssets<Mesh<Vertex3DTex>>>().unwrap();
    let Some(mesh) = gpu_meshes.get(&mesh_handle.id()) else {
        return RenderResult::Failure;
    };
    // -- -- -- -------- -- -- --

    // -- Bind Model, View, Texture BindGroups --
    let mesh3d_bind_groups = world.get_resource::<Mesh3DBindGroups>().unwrap();

    let model_uniform_id = world.get::<DynamicUniformId<ModelUniform>>(object).unwrap();
    render_pass.set_bind_group(
        0,
        mesh3d_bind_groups.model_bind_group.as_ref().unwrap(),
        &[**model_uniform_id],
    );

    let view_uniform_id = world
        .get::<DynamicUniformId<CameraUniforms>>(camera)
        .unwrap();
    render_pass.set_bind_group(
        1,
        mesh3d_bind_groups.view_bind_group.as_ref().unwrap(),
        &[**view_uniform_id],
    );

    // let texture_bind_groups = world.get_resource::<TextureBindGroups>().unwrap();
    // let texture_bind_group = match world.get::<Textures>(object) {
    //     Some(image_handle) => match texture_bind_groups.get(&image_handle.id()) {
    //         Some(bind) => bind,
    //         None => &sprite_pipeline.dummy_texture_bind_group,
    //     },
    //     None => &sprite_pipeline.dummy_texture_bind_group,
    // };
    let texture_bind_group = &mesh3d_pipeline.dummy_arr_texture_bind_group;
    render_pass.set_bind_group(2, texture_bind_group, &[]);
    // -- -- -- -------- -- -- --

    // -- Set Mesh Buffers --
    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    let instance_count = 1;
    // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    match &mesh.assembly {
        GpuMeshAssembly::Indexed {
            index_buffer,
            index_count,
            index_format,
        } => {
            render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
            render_pass.draw_indexed(0..*index_count as u32, 0, 0..instance_count);
        }
        GpuMeshAssembly::NonIndexed { vertex_count } => {
            render_pass.draw(0..*vertex_count as u32, 0..instance_count);
        }
    }
    // -- -- -- -------- -- -- --

    RenderResult::Success
}
