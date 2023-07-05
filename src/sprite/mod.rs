use bevy::{
    asset::load_internal_asset,
    prelude::{Assets, Entity, Handle, HandleUntyped, Plugin, World},
    reflect::TypeUuid,
};

use crate::{
    core_pipeline::bind::CommonBindGroups,
    render::{
        camera::component::CameraUniforms,
        mesh::{primitive::{quad::{create_unit_square_colored, create_unit_square}, triangle::create_unit_triangle}, GpuMeshAssembly, Mesh},
        resource::{
            buffer::{Vertex, VertexC, VertexBase},
            component_uniform::ModelUniform,
            pipeline::PipelineCache,
            shader::Shader,
            uniform::DynamicUniformId,
        },
        system::{AddRenderFunction, RenderResult},
        texture::Image,
        RenderAssets, RenderStage, uniform::{RadiusUniform, ColorUniform},
    },
    sprite::bind::{create_texture_bind_groups, SpritePipeline, TextureBindGroups, create_circle_bind_groups, create_triangle_bind_groups},
};

use self::bind::{CirclePipeline, CircleBindGroups, TrianglePipeline, TriangleBindGroups};

pub mod bind;
pub mod bundle;

const SPRITE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 45678909876445673);

const CIRCLE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 45678909876445674);

const TRIANGLE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 45678909876445675);

pub const BASE_QUAD_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::<Vertex>::TYPE_UUID, 45678909876445676);

pub const BASE_QUAD_COLORED_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::<VertexC>::TYPE_UUID, 45678909876445677);

pub const BASE_TRIANGLE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::<VertexBase>::TYPE_UUID, 45678909876445678);

pub struct FlatSpritePlugin;
impl Plugin for FlatSpritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, SPRITE_SHADER_HANDLE, "sprite.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, CIRCLE_SHADER_HANDLE, "circle.wgsl", Shader::from_wgsl);
        load_internal_asset!(app, TRIANGLE_SHADER_HANDLE, "triangle.wgsl", Shader::from_wgsl);

        {
            let mut meshes = app
                .world
                .get_resource_mut::<Assets<Mesh<VertexC>>>()
                .unwrap();
            meshes.set_untracked(BASE_QUAD_COLORED_HANDLE, create_unit_square_colored());
        }

        {
            let mut meshes = app
                .world
                .get_resource_mut::<Assets<Mesh<Vertex>>>()
                .unwrap();
            meshes.set_untracked(BASE_QUAD_HANDLE, create_unit_square());
        }

        {
            let mut meshes = app
                .world
                .get_resource_mut::<Assets<Mesh<VertexBase>>>()
                .unwrap();
            meshes.set_untracked(BASE_TRIANGLE_HANDLE, create_unit_triangle());
        }

        app.init_resource::<SpritePipeline>()
            .init_resource::<CirclePipeline>()
            .init_resource::<TrianglePipeline>()
            .init_resource::<CircleBindGroups>()
            .init_resource::<TriangleBindGroups>()
            .init_resource::<TextureBindGroups>()
            .add_render_function(SPRITE_RENDER_FUNCTION, render_sprite)
            .add_render_function(CIRCLE_RENDER_FUNCTION, render_circle)
            .add_render_function(TRIANGLE_RENDER_FUNCTION, render_triangle)
            .add_system_to_stage(RenderStage::Create, create_circle_bind_groups)
            .add_system_to_stage(RenderStage::Create, create_triangle_bind_groups)
            .add_system_to_stage(RenderStage::Create, create_texture_bind_groups);
    }
}

pub const SPRITE_RENDER_FUNCTION: usize = 1;
fn render_sprite<'w>(
    camera: Entity,
    object: Entity,
    world: &'w World,
    render_pass: &mut wgpu::RenderPass<'w>,
) -> RenderResult {
    // -- Set Pipeline --
    let sprite_pipeline = world.get_resource::<SpritePipeline>().unwrap();
    let pipeline_cache = world.get_resource::<PipelineCache>().unwrap();
    let Some(render_pipeline) = pipeline_cache.get(&sprite_pipeline.pipeline_id) else {
        return RenderResult::Failure;
    };
    render_pass.set_pipeline(render_pipeline);
    // -- -- -- -------- -- -- --

    // -- Get Mesh --
    let Some(mesh_handle) = world.get::<Handle<Mesh<VertexC>>>(object) else {
        return RenderResult::Failure;
    };
    let gpu_meshes = world.get_resource::<RenderAssets<Mesh<VertexC>>>().unwrap();
    let Some(mesh) = gpu_meshes.get(&mesh_handle.id()) else {
        return RenderResult::Failure;
    };
    // -- -- -- -------- -- -- --

    // -- Bind Model, View, Texture BindGroups --
    let common_bind_groups = world.get_resource::<CommonBindGroups>().unwrap();

    let model_uniform_id = world.get::<DynamicUniformId<ModelUniform>>(object).unwrap();
    render_pass.set_bind_group(
        0,
        common_bind_groups.model_bind_group.as_ref().unwrap(),
        &[**model_uniform_id],
    );

    let view_uniform_id = world
        .get::<DynamicUniformId<CameraUniforms>>(camera)
        .unwrap();
    render_pass.set_bind_group(
        1,
        common_bind_groups.view_bind_group.as_ref().unwrap(),
        &[**view_uniform_id],
    );

    let texture_bind_groups = world.get_resource::<TextureBindGroups>().unwrap();
    let texture_bind_group = match world.get::<Handle<Image>>(object) {
        Some(image_handle) => match texture_bind_groups.get(&image_handle.id()) {
            Some(bind) => bind,
            None => &sprite_pipeline.dummy_texture_bind_group,
        },
        None => &sprite_pipeline.dummy_texture_bind_group,
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

pub const CIRCLE_RENDER_FUNCTION: usize = 3;
fn render_circle<'w>(
    camera: Entity,
    object: Entity,
    world: &'w World,
    render_pass: &mut wgpu::RenderPass<'w>,
) -> RenderResult {
    // -- Set Pipeline --
    let circle_pipeline = world.get_resource::<CirclePipeline>().unwrap();
    let pipeline_cache = world.get_resource::<PipelineCache>().unwrap();
    let Some(render_pipeline) = pipeline_cache.get(&circle_pipeline.pipeline_id) else {
        return RenderResult::Failure;
    };
    render_pass.set_pipeline(render_pipeline);
    // -- -- -- -------- -- -- --

    // -- Get Mesh --
    // Mesh: BASE_QUAD
    let gpu_meshes = world.get_resource::<RenderAssets<Mesh<Vertex>>>().unwrap();
    let Some(mesh) = gpu_meshes.get(&BASE_QUAD_HANDLE.id) else {
        return RenderResult::Failure;
    };
    // -- -- -- -------- -- -- --

    // -- Bind Model, View, Radius, Color BindGroups --
    let common_bind_groups = world.get_resource::<CommonBindGroups>().unwrap();
    let circle_bind_groups = world.get_resource::<CircleBindGroups>().unwrap();

    let model_uniform_id = world.get::<DynamicUniformId<ModelUniform>>(object).unwrap();
    render_pass.set_bind_group(
        0,
        common_bind_groups.model_bind_group.as_ref().unwrap(),
        &[**model_uniform_id],
    );

    let view_uniform_id = world
        .get::<DynamicUniformId<CameraUniforms>>(camera)
        .unwrap();
    render_pass.set_bind_group(
        1,
        common_bind_groups.view_bind_group.as_ref().unwrap(),
        &[**view_uniform_id],
    );

    let radius_uniform_id = world
        .get::<DynamicUniformId<RadiusUniform>>(object)
        .unwrap();
    render_pass.set_bind_group(
        2,
        circle_bind_groups.radius_bind_group.as_ref().unwrap(),
        &[**radius_uniform_id],
    );

    let color_uniform_id = world
        .get::<DynamicUniformId<ColorUniform>>(object)
        .unwrap();
    render_pass.set_bind_group(
        3,
        circle_bind_groups.color_bind_group.as_ref().unwrap(),
        &[**color_uniform_id],
    );
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

pub const TRIANGLE_RENDER_FUNCTION: usize = 4;
fn render_triangle<'w>(
    camera: Entity,
    object: Entity,
    world: &'w World,
    render_pass: &mut wgpu::RenderPass<'w>,
) -> RenderResult {
    // -- Set Pipeline --
    let triangle_pipeline = world.get_resource::<TrianglePipeline>().unwrap();
    let pipeline_cache = world.get_resource::<PipelineCache>().unwrap();
    let Some(render_pipeline) = pipeline_cache.get(&triangle_pipeline.pipeline_id) else {
        return RenderResult::Failure;
    };
    render_pass.set_pipeline(render_pipeline);
    // -- -- -- -------- -- -- --

    // -- Get Mesh --
    let Some(mesh_handle) = world.get::<Handle<Mesh<VertexBase>>>(object) else {
        return RenderResult::Failure;
    };
    let gpu_meshes = world.get_resource::<RenderAssets<Mesh<VertexBase>>>().unwrap();
    let Some(mesh) = gpu_meshes.get(&mesh_handle.id()) else {
        return RenderResult::Failure;
    };
    // -- -- -- -------- -- -- --

    // -- Bind Model, View, Radius, Color BindGroups --
    let common_bind_groups = world.get_resource::<CommonBindGroups>().unwrap();
    let triangle_bind_groups = world.get_resource::<TriangleBindGroups>().unwrap();

    let model_uniform_id = world.get::<DynamicUniformId<ModelUniform>>(object).unwrap();
    render_pass.set_bind_group(
        0,
        common_bind_groups.model_bind_group.as_ref().unwrap(),
        &[**model_uniform_id],
    );

    let view_uniform_id = world
        .get::<DynamicUniformId<CameraUniforms>>(camera)
        .unwrap();
    render_pass.set_bind_group(
        1,
        common_bind_groups.view_bind_group.as_ref().unwrap(),
        &[**view_uniform_id],
    );

    let color_uniform_id = world
        .get::<DynamicUniformId<ColorUniform>>(object)
        .unwrap();
    render_pass.set_bind_group(
        2,
        triangle_bind_groups.color_bind_group.as_ref().unwrap(),
        &[**color_uniform_id],
    );
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