use bevy::prelude::{Bundle, GlobalTransform, Handle, Transform};

use crate::render::{
    color::Color, mesh::Mesh, resource::buffer::Vertex, system::RenderFunctionId, texture::Image, camera::component::Visibility,
};

use super::SPRITE_RENDER_FUNCTION;

#[derive(Bundle)]
pub struct SpriteBundle {
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub mesh: Handle<Mesh<Vertex>>,
    pub texture: Handle<Image>,
    pub color: Color,
    pub visibility: Visibility,
    pub render_function: RenderFunctionId,
}

impl Default for SpriteBundle {
    fn default() -> Self {
        Self {
            global_transform: GlobalTransform::default(),
            transform: Transform::default(),
            mesh: Handle::default(),
            texture: Handle::default(),
            color: Color::WHITE,
            visibility: Visibility { visible: true },
            render_function: SPRITE_RENDER_FUNCTION.into(),
        }
    }
}
