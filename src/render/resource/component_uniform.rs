use bevy::prelude::{
    App, Commands, Component, Deref, DerefMut, Entity, GlobalTransform, Mat4, Query, Res, ResMut,
    Resource,
};
use encase::{private::WriteInto, ShaderType};

use crate::render::RenderStage;

use super::{
    renderer::{RenderDevice, RenderQueue},
    uniform::{DynamicUniformBuffer, DynamicUniformId, HandleGpuUniform},
};

#[derive(Resource, Deref, DerefMut)]
pub struct ComponentUniforms<T: ShaderType + WriteInto + Send + Sync + 'static>(
    pub DynamicUniformBuffer<T>,
);
impl<T: ShaderType + WriteInto + Send + Sync + 'static> Default for ComponentUniforms<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait AddComponentUniform {
    fn add_component_uniform<H: HandleGpuUniform + Component>(&mut self) -> &mut Self;
}
impl AddComponentUniform for App {
    fn add_component_uniform<H: HandleGpuUniform + Component>(&mut self) -> &mut Self {
        self.init_resource::<ComponentUniforms<H::GU>>()
            .add_system_to_stage(RenderStage::Prepare, prepare_component_uniforms::<H>)
            .add_system_to_stage(RenderStage::Create, queue_component_uniforms::<H>)
    }
}

pub fn prepare_component_uniforms<H: HandleGpuUniform + Component>(
    mut commands: Commands,
    mut component_uniforms: ResMut<ComponentUniforms<H::GU>>,
    query: Query<(Entity, &H)>,
) {
    let mut spawns: Vec<(Entity, DynamicUniformId<H::GU>)> = Vec::new();

    component_uniforms.clear();
    for (entity, uniform_handle) in query.iter() {
        spawns.push((
            entity,
            component_uniforms
                .push(uniform_handle.into_uniform())
                .into(),
        ));
    }

    for (entity, _) in &spawns {
        commands.entity(*entity).remove::<DynamicUniformId<H::GU>>();
    }
    commands.insert_or_spawn_batch(spawns);
}

pub fn queue_component_uniforms<H: HandleGpuUniform + Component>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut component_uniforms: ResMut<ComponentUniforms<H::GU>>,
) {
    component_uniforms.write_buffer(&render_device, &render_queue);
}

#[derive(Clone, ShaderType)]
pub struct ModelUniform {
    model: Mat4,
}

impl HandleGpuUniform for GlobalTransform {
    type GU = ModelUniform;

    fn into_uniform(&self) -> Self::GU {
        ModelUniform {
            model: self.compute_matrix(),
        }
    }
}
