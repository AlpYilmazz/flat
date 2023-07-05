use bevy::prelude::{Bundle, GlobalTransform, Handle, Transform};

use crate::render::{
    uniform::{Color, Radius}, mesh::Mesh, resource::buffer::{VertexC, VertexBase}, system::RenderFunctionId, texture::Image, camera::component::Visibility,
};

use super::{SPRITE_RENDER_FUNCTION, CIRCLE_RENDER_FUNCTION, TRIANGLE_RENDER_FUNCTION};

#[derive(Bundle)]
pub struct SpriteBundle {
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub mesh: Handle<Mesh<VertexC>>,
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

#[derive(Bundle)]
pub struct SimpleCircleBundle {
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub radius: Radius,
    pub color: Color,
    pub visibility: Visibility,
    pub render_function: RenderFunctionId,
}

impl Default for SimpleCircleBundle {
    fn default() -> Self {
        Self {
            global_transform: GlobalTransform::default(),
            transform: Transform::default(),
            radius: Radius(0.5),
            color: Color::WHITE,
            visibility: Visibility { visible: true },
            render_function: CIRCLE_RENDER_FUNCTION.into(),
        }
    }
}

#[derive(Bundle)]
pub struct SimpleTriangleBundle {
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub mesh: Handle<Mesh<VertexBase>>,
    pub color: Color,
    pub visibility: Visibility,
    pub render_function: RenderFunctionId,
}

impl Default for SimpleTriangleBundle {
    fn default() -> Self {
        Self {
            global_transform: GlobalTransform::default(),
            transform: Transform::default(),
            mesh: Handle::default(),
            color: Color::WHITE,
            visibility: Visibility { visible: true },
            render_function: TRIANGLE_RENDER_FUNCTION.into(),
        }
    }
}