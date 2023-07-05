use bevy::prelude::{Component, Vec4};
use encase::ShaderType;

use super::resource::uniform::HandleGpuUniform;

#[derive(Component, Clone, Copy)]
pub struct Radius(pub f32);

#[derive(Clone, ShaderType)]
pub struct RadiusUniform {
    value: f32,
}

impl HandleGpuUniform for Radius {
    type GU = RadiusUniform;

    fn into_uniform(&self) -> Self::GU {
        RadiusUniform { value: self.0 }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

impl Color {
    pub const WHITE: Color = Color(0.0, 0.0, 0.0, 1.0);

    pub fn as_vec(&self) -> Vec4 {
        Vec4::new(self.0, self.1, self.2, self.3)
    }

    pub fn as_arr(&self) -> [f32; 4] {
        [self.0, self.1, self.2, self.3]
    }

    #[inline]
    pub fn r(&self) -> f32 {
        self.0
    }

    #[inline]
    pub fn g(&self) -> f32 {
        self.1
    }

    #[inline]
    pub fn b(&self) -> f32 {
        self.2
    }

    #[inline]
    pub fn a(&self) -> f32 {
        self.3
    }
}

#[derive(Clone, ShaderType)]
pub struct ColorUniform {
    color: Vec4,
}

impl HandleGpuUniform for Color {
    type GU = ColorUniform;

    fn into_uniform(&self) -> Self::GU {
        ColorUniform {
            color: self.as_vec(),
        }
    }
}
