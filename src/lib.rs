use bevy::{
    app::PluginGroupBuilder,
    prelude::{App, Plugin, PluginGroup},
    DefaultPlugins,
};
use mesh3d::FlatMeshPlugin;
use render::FlatRenderPlugin;
use sprite::FlatSpritePlugin;

pub mod mesh3d;
pub mod render;
pub mod shapes;
pub mod sprite;

pub mod misc;
pub mod text;
pub mod util;

/*
TypeUuid

6948DF80-14BD-4E04-8842-7668D9C001F5 - Text
4B8302DA-21AD-401F-AF45-1DFD956B80B5 - Shader
8628FE7C-A4E9-4056-91BD-FD6AA7817E39 - Mesh<V: MeshVertex>
ED280816-E404-444A-A2D9-FFD2D171F928 - BatchMesh<V: MeshVertex>
D952EB9F-7AD2-4B1B-B3CE-386735205990 - Quad
3F897E85-62CE-4B2C-A957-FCF0CCE649FD - Image
8E7C2F0A-6BB8-485C-917E-6B605A0DDF29 - ImageArray
1AD2F3EF-87C8-46B4-BD1D-94C174C278EE
AA97B177-9383-4934-8543-0F91A7A02836 - Vertex3Tex: MeshVertex
10929DF8-15C5-472B-9398-7158AB89A0A6 - Vertex: MeshVertex
*/

pub struct FlatEngineComplete;

impl PluginGroup for FlatEngineComplete {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(FlatBevyPlugins)
            .add(FlatEngineCore)
    }
}

pub struct FlatBevyPlugins;
impl Plugin for FlatBevyPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugin(BevyPluginSettings);

        // app.add_plugin(bevy::log::LogPlugin::default())
        //     .add_plugin(bevy::core::CorePlugin::default())
        //     .add_plugin(bevy::time::TimePlugin::default())
        //     .add_plugin(bevy::transform::TransformPlugin::default())
        //     .add_plugin(bevy::hierarchy::HierarchyPlugin::default())
        //     .add_plugin(bevy::diagnostic::DiagnosticsPlugin::default())
        //     .add_plugin(bevy::input::InputPlugin::default())
        //     .add_plugin(bevy::window::WindowPlugin {
        //         window: Default::default(),
        //         add_primary_window: true,
        //         exit_on_all_closed: true,
        //         close_when_requested: true,
        //     });

        // app.add_plugin(bevy::winit::WinitPlugin::default())
        //     .add_plugin(bevy::asset::AssetPlugin {
        //         asset_folder: "res".to_string(),
        //         watch_for_changes: false,
        //     });

        app.add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..Default::default()
                })
                .set(bevy::window::WindowPlugin {
                    window: Default::default(),
                    add_primary_window: true,
                    exit_on_all_closed: true,
                    close_when_requested: true,
                })
                .set(bevy::asset::AssetPlugin {
                    asset_folder: "res".to_string(),
                    watch_for_changes: false,
                }), // .disable::<bevy::render::RenderPlugin>()
        );
    }
}

pub struct BevyPluginSettings;
impl Plugin for BevyPluginSettings {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy::winit::WinitSettings::game());
    }
}

pub struct FlatEngineCore;
impl Plugin for FlatEngineCore {
    fn build(&self, app: &mut App) {
        app.add_plugin(FlatRenderPlugin)
            .add_plugin(FlatSpritePlugin)
            .add_plugin(FlatMeshPlugin);
    }
}
