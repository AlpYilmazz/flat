use bevy::prelude::Plugin;

use crate::render::RenderStage;

use self::bind::{create_common_bind_groups, CommonBindGroups, PipelineCommons};

pub mod bind;

pub struct FlatCorePipelinePlugin;
impl Plugin for FlatCorePipelinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<PipelineCommons>()
            .init_resource::<CommonBindGroups>()
            .add_system_to_stage(RenderStage::Create, create_common_bind_groups);
    }
}
