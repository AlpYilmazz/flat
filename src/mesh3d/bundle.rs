use bevy::prelude::{Bundle, GlobalTransform, Handle, Transform};

use crate::render::{
    camera::component::Visibility, color::Color, mesh::Mesh, resource::buffer::MeshVertex,
    system::RenderFunctionId, texture::texture_arr::ImageArrayHandle,
};

use super::{bind::MeshPipelineKey, MESH_RENDER_FUNCTION};

#[derive(Bundle)]
pub struct MeshBundle<V: MeshVertex> {
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub mesh: Handle<Mesh<V>>,
    pub textures: ImageArrayHandle,
    pub color: Color,
    pub visibility: Visibility,
    pub render_key: MeshPipelineKey,
    pub render_function: RenderFunctionId,
}

impl<V: MeshVertex> Default for MeshBundle<V> {
    fn default() -> Self {
        Self {
            global_transform: GlobalTransform::default(),
            transform: Transform::default(),
            mesh: Handle::default(),
            textures: ImageArrayHandle::default(),
            color: Color::WHITE,
            visibility: Visibility { visible: true },
            render_key: MeshPipelineKey { texture_count: 1 },
            render_function: MESH_RENDER_FUNCTION.into(),
        }
    }
}
