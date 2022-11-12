use bevy_app::Plugin;
use bevy_ecs::{system::{lifetimeless::{SQuery, Read, SRes}, SystemParamItem}, prelude::Entity};
use bevy_render::render_phase::{SetItemPipeline, AddRenderCommand, RenderCommand, BatchedPhaseItem, TrackedRenderPass};

use crate::core_2d::PrimitiveQuad;

pub struct FlatSpritePlugin;
impl Plugin for FlatSpritePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_render_command::<PrimitiveQuad, DrawSprite>();
    }
}

type DrawSprite = (
    SetItemPipeline,
    DrawSpriteSingle,
);

pub struct DrawSpriteSingle;
impl RenderCommand<PrimitiveQuad> for DrawSpriteSingle {
    type Param = (SRes<>, SQuery<Read<SpriteBatch>>);

    fn render<'w>(
        _view: Entity,
        item: &PrimitiveQuad,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> bevy_render::render_phase::RenderCommandResult {
        todo!()
    }
}