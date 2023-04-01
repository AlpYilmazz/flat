use bevy::{
    asset::load_internal_asset,
    prelude::{Entity, Handle, HandleUntyped, Plugin, World},
    reflect::TypeUuid,
};

use crate::{
    mesh3d::bind::{
        create_mesh3d_bind_groups, create_texture_arr_bind_groups, MeshBindGroups, MeshPipeline,
    },
    render::{
        camera::component::CameraUniforms,
        mesh::{GpuMeshAssembly, Mesh},
        resource::{
            buffer::VertexTex3, component_uniform::ModelUniform, pipeline::PipelineCache,
            shader::Shader, specialized_pipeline::Specialized, uniform::DynamicUniformId,
        },
        system::{AddRenderFunction, RenderResult},
        texture::texture_arr::ImageArrayHandle,
        RenderAssets, RenderStage,
    },
};

use self::bind::{MeshPipelineKey, TextureArrayBindGroups};

pub mod bind;
pub mod bundle;

const MESH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 15678909876445673);

// pub const BASE_CUBE_HANDLE: HandleUntyped =
//     HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 15678909876445674);

pub struct FlatMeshPlugin;
impl Plugin for FlatMeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            MESH_SHADER_HANDLE,
            "mesh_texarr.wgsl",
            Shader::from_wgsl
        );

        // {
        //     let mut meshes = app
        //         .world
        //         .get_resource_mut::<Assets<Mesh<VertexTex3>>>()
        //         .unwrap();
        //     meshes.set_untracked(BASE_CUBE_HANDLE, create_unit_cube(FaceDirection::Out));
        // }

        app.init_resource::<Specialized<MeshPipeline>>()
            .init_resource::<MeshPipeline>()
            .init_resource::<MeshBindGroups>()
            .init_resource::<TextureArrayBindGroups>()
            .add_render_function(MESH_RENDER_FUNCTION, render_mesh)
            .add_system_to_stage(RenderStage::Create, create_mesh3d_bind_groups)
            .add_system_to_stage(RenderStage::Create, create_texture_arr_bind_groups);
    }
}

const MESH_RENDER_FUNCTION: usize = 2;
fn render_mesh<'w>(
    camera: Entity,
    object: Entity,
    world: &'w World,
    render_pass: &mut wgpu::RenderPass<'w>,
) -> RenderResult {
    // -- Set Pipeline --
    let mesh_pipeline = world.get_resource::<MeshPipeline>().unwrap();
    let specialized_mesh_pipeline = world.get_resource::<Specialized<MeshPipeline>>().unwrap();
    let pipeline_cache = world.get_resource::<PipelineCache>().unwrap();

    let Some(pipeline_key) = world.get::<MeshPipelineKey>(object) else {
        return RenderResult::Failure;
    };
    let Some(pipeline_id) = specialized_mesh_pipeline.pipelines.get(pipeline_key) else {
        return RenderResult::Failure;
    };
    let Some(render_pipeline) = pipeline_cache.get(pipeline_id) else {
        return RenderResult::Failure;
    };
    render_pass.set_pipeline(render_pipeline);
    // -- -- -- -------- -- -- --

    // -- Get Mesh --
    let Some(mesh_handle) = world.get::<Handle<Mesh<VertexTex3>>>(object) else {
        return RenderResult::Failure;
    };
    let gpu_meshes = world
        .get_resource::<RenderAssets<Mesh<VertexTex3>>>()
        .unwrap();
    let Some(mesh) = gpu_meshes.get(&mesh_handle.id()) else {
        return RenderResult::Failure;
    };
    // -- -- -- -------- -- -- --

    // -- Bind Model, View, Texture BindGroups --
    let mesh3d_bind_groups = world.get_resource::<MeshBindGroups>().unwrap();

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

    let texture_array_bind_groups = world.get_resource::<TextureArrayBindGroups>().unwrap();
    let texture_bind_group = match world.get::<ImageArrayHandle>(object) {
        Some(image_array_handle) => match &image_array_handle.image_arr {
            Some(handle) => match texture_array_bind_groups.get(&handle.id()) {
                Some(bind) => bind,
                None => &mesh_pipeline.dummy_texture_arr_bind_group,
            },
            None => &mesh_pipeline.dummy_texture_arr_bind_group,
        },
        None => &mesh_pipeline.dummy_texture_arr_bind_group,
    };
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
