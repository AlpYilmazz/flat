use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::{
    render::resource::{
        bind::BindingSet,
        uniform::{HandleGpuUniform, UniformDesc},
    },
    transform::GlobalTransform,
    util::{store, Refer, Sink, Store},
};

use super::{resource::uniform::Uniform, RenderStage};

pub struct RenderTransformPlugin;
impl Plugin for RenderTransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(RenderStage::Extract, extract_global_transforms);
    }
}

#[derive(Component)]
pub struct ExtractedTransform {
    pub uniform: Uniform<GlobalTransform>,
    pub bind_refer: Refer<wgpu::BindGroup>,
}

pub fn extract_global_transforms(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    mut query: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&mut ExtractedTransform>,
            Added<GlobalTransform>,
            Changed<GlobalTransform>,
        ),
        Or<(Added<GlobalTransform>, Changed<GlobalTransform>)>,
    >,
    removed: RemovedComponents<GlobalTransform>,
    mut bind_groups: ResMut<Store<wgpu::BindGroup>>,
    mut commands: Commands,
) {
    for (entity, gtransform, extracted_transform, added, changed) in query.iter_mut() {
        if added {
            let uniform = Uniform::new(&device, gtransform.generate_uniform());
            let bind_group = uniform
                .as_ref()
                .into_bind_group(&device, &UniformDesc::default());
            let bind_refer = store(&mut bind_groups, bind_group);
            commands.entity(entity).insert(ExtractedTransform {
                uniform,
                bind_refer,
            });
        } else if changed {
            let uniform = &mut extracted_transform.unwrap().uniform;
            gtransform.update_uniform(&mut uniform.gpu_uniform);
            uniform.sync_buffer(&queue);
        };
    }

    removed.iter().for_each(|entity| {
        commands
            .entity(entity)
            .remove::<ExtractedTransform>()
            .sink()
    });
}
