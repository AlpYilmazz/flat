use bevy_app::{Plugin, CoreStage};
use bevy_ecs::{
    prelude::{Component, Entity},
    query::{Changed, With, Without},
    system::Query,
};
use bytemuck::{Pod, Zeroable};
use cgmath::{Vector3, Quaternion, Matrix4, Zero, One, ElementWise, SquareMatrix};
use repr_trait::C;

use crate::{hierarchy::{Children, Parent}, render::resource::uniform::{GpuUniform, HandleGpuUniform}};

pub struct FlatTransformPlugin;
impl Plugin for FlatTransformPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_system_to_stage(CoreStage::PostUpdate, transform_propagate_system);
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn compose(base: &Transform, transform: &Transform) -> Transform {
        Transform {
            translation: base.translation + transform.translation,
            rotation: base.rotation * transform.rotation,
            scale: base.scale.mul_element_wise(transform.scale),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct GlobalTransform(pub Transform);

impl GlobalTransform {
    pub fn update_from(&mut self, base: &GlobalTransform, transform: &Transform) {
        self.0 = Transform::compose(&base.0, transform);
    }

    pub fn compose(base: &GlobalTransform, transform: &Transform) -> GlobalTransform {
        GlobalTransform(Transform::compose(&base.0, transform))
    }
}

impl HandleGpuUniform for GlobalTransform {
    type GU = ModelMatrix;

    fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
        gpu_uniform.model = (
            Matrix4::from_translation(self.0.translation)
            * Matrix4::from_nonuniform_scale(self.0.scale.x, self.0.scale.y, self.0.scale.z)
            * Matrix4::from(self.0.rotation)
        ).into()
    }
}

impl From<Transform> for GlobalTransform {
    fn from(val: Transform) -> Self {
        Self(val)
    }
}

#[repr(C)]
#[derive(Component, Debug, Clone, Copy, C, Pod, Zeroable)]
pub struct ModelMatrix {
    pub model: [[f32; 4]; 4],
}
impl GpuUniform for ModelMatrix {
    const STAGE: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX;
}
impl Default for ModelMatrix {
    fn default() -> Self {
        Self {
            model: Matrix4::identity().into(),
        }
    }
}

pub fn transform_propagate_system(
    mut root_query: Query<
        (
            Option<&Children>,
            &Transform,
            Changed<Transform>,
            &mut GlobalTransform,
        ),
        Without<Parent>,
    >,
    mut transform_query: Query<
        (&Transform, Changed<Transform>, &mut GlobalTransform),
        With<Parent>,
    >,
    children_query: Query<Option<&Children>, (With<Parent>, With<GlobalTransform>)>,
) {
    for (children, transform, transform_changed, mut global_transform) in root_query.iter_mut() {
        if transform_changed {
            global_transform.0 = transform.clone();
        }

        if let Some(children) = children {
            for child in children.as_ref() {
                propagate_recursive(
                    &global_transform,
                    &mut transform_query,
                    &children_query,
                    *child,
                    transform_changed,
                );
            }
        }
    }
}

fn propagate_recursive(
    parent_global_transform: &GlobalTransform,
    transform_query: &mut Query<
        (&Transform, Changed<Transform>, &mut GlobalTransform),
        With<Parent>,
    >,
    children_query: &Query<Option<&Children>, (With<Parent>, With<GlobalTransform>)>,
    entity: Entity,
    mut changed: bool,
) {
    let global_matrix = {
        if let Ok((transform, transform_changed, mut global_transform)) =
            transform_query.get_mut(entity)
        {
            changed |= transform_changed;
            if changed {
                global_transform.update_from(parent_global_transform, transform);
            }
            global_transform.clone()
        } else {
            return;
        }
    };

    if let Ok(Some(children)) = children_query.get(entity) {
        for child in children.as_ref() {
            propagate_recursive(
                &global_matrix,
                transform_query,
                children_query,
                *child,
                changed,
            );
        }
    }
}
