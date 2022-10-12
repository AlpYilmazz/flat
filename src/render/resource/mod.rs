use bevy_ecs::{
    prelude::{Component, Entity},
    query::{Added, Changed, Or},
    system::{Query, Res, ResMut},
};

use crate::util::EntityStore;

use self::uniform::{Uniform, HandleGpuUniform};

pub mod bind;
pub mod buffer;
pub mod pipeline;
pub mod shader;
pub mod uniform;

pub fn extract_uniform<T>(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    query: Query<(Entity, &T, Added<T>, Changed<T>), Or<(Added<T>, Changed<T>)>>,
    mut uniforms: ResMut<EntityStore<Uniform<T>>>,
) where
    T: HandleGpuUniform + Component,
    T::GU: Default,
{
    for (entity, uniform_handle, added, changed) in query.iter() {
        if added {
            let uniform = Uniform::new(&device, uniform_handle.generate_uniform());
            uniforms.insert(entity, uniform);
        } else if changed {
            let uniform = uniforms.get_mut(&entity).unwrap();
            uniform.sync_buffer(&queue);
        };
    }
}
