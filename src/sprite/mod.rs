use bevy::{
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    prelude::*,
    render::render_phase::{
        AddRenderCommand, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
    },
};

use crate::core_2d::PrimitiveQuad;

pub struct FlatSpritePlugin;
impl Plugin for FlatSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_render_command::<PrimitiveQuad, DrawSprite>();
    }
}

type DrawSprite = (SetItemPipeline, DrawSpriteSingle);

pub struct DrawSpriteSingle;
impl RenderCommand<PrimitiveQuad> for DrawSpriteSingle {
    type Param = (); //(SRes<()>, SQuery<Read<SpriteBatch>>);

    fn render<'w>(
        _view: Entity,
        item: &PrimitiveQuad,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        todo!()
    }
}
