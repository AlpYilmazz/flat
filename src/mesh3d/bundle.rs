use bevy::prelude::{Bundle, GlobalTransform, Transform, Handle, Component, Deref, DerefMut};

use crate::render::{mesh::Mesh, resource::buffer::MeshVertex, color::Color, camera::component::Visibility, system::RenderFunctionId, texture::Image};

use super::MESH3D_RENDER_FUNCTION;

#[derive(Bundle)]
pub struct Mesh3DBundle<V: MeshVertex> {
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub mesh: Handle<Mesh<V>>,
    pub textures: Textures,
    pub color: Color,
    pub visibility: Visibility,
    pub render_function: RenderFunctionId,
}

impl<V: MeshVertex> Default for Mesh3DBundle<V> {
    fn default() -> Self {
        Self {
            global_transform: GlobalTransform::default(),
            transform: Transform::default(),
            mesh: Handle::default(),
            textures: Textures(Vec::new()),
            color: Color::WHITE,
            visibility: Visibility { visible: true },
            render_function: MESH3D_RENDER_FUNCTION.into(),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Textures(pub Vec<Handle<Image>>);