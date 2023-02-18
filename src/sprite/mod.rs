use bevy::{prelude::{Plugin, HandleUntyped}, reflect::TypeUuid, asset::load_internal_asset};

use crate::{
    render::{RenderStage, resource::shader::Shader},
    sprite::bind::{create_texture_bind_groups, SpritePipeline, TextureBindGroups},
};

pub mod bind;

const SPRITE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 45678909876445673);

pub struct FlatSpritePlugin;
impl Plugin for FlatSpritePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, SPRITE_SHADER_HANDLE, "sprite.wgsl", Shader::from_wgsl);

        app.init_resource::<SpritePipeline>()
            .init_resource::<TextureBindGroups>()
            .add_system_to_stage(RenderStage::Create, create_texture_bind_groups);
    }
}
