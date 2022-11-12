use bevy_app::{Plugin, PluginGroup};

pub mod core_2d;
pub mod sprite;

pub mod misc;
pub mod text;
pub mod util;

/*
TypeUuid

6948DF80-14BD-4E04-8842-7668D9C001F5 - Text
4B8302DA-21AD-401F-AF45-1DFD956B80B5 - ShaderSource
8628FE7C-A4E9-4056-91BD-FD6AA7817E39 - Mesh<V: MeshVertex>
ED280816-E404-444A-A2D9-FFD2D171F928 - BatchMesh<V: MeshVertex>
D952EB9F-7AD2-4B1B-B3CE-386735205990 - Quad
3F897E85-62CE-4B2C-A957-FCF0CCE649FD - Image
8E7C2F0A-6BB8-485C-917E-6B605A0DDF29
1AD2F3EF-87C8-46B4-BD1D-94C174C278EE
AA97B177-9383-4934-8543-0F91A7A02836
10929DF8-15C5-472B-9398-7158AB89A0A6 - Vertex: MeshVertex
*/

pub struct FlatEngineComplete;

impl PluginGroup for FlatEngineComplete {
    fn build(&mut self, group: &mut bevy_app::PluginGroupBuilder) {
        BevyPlugins.build(group);
        FlatEngineCore.build(group);
    }
}

pub struct BevyPlugins;
impl PluginGroup for BevyPlugins {
    fn build(&mut self, group: &mut bevy_app::PluginGroupBuilder) {
        group.add(BevyPluginSettings);

        group
            .add(bevy_log::LogPlugin)
            .add(bevy_core::CorePlugin)
            .add(bevy_time::TimePlugin)
            .add(bevy_transform::TransformPlugin)
            .add(bevy_hierarchy::HierarchyPlugin)
            // .add(bevy_diagnostic::DiagnosticsPlugin::default())
            .add(bevy_input::InputPlugin)
            .add(bevy_window::WindowPlugin);

        group
            .add(bevy_asset::AssetPlugin)
            .add(bevy_winit::WinitPlugin)
            .add(bevy_render::RenderPlugin);
    }
}

pub struct BevyPluginSettings;
impl Plugin for BevyPluginSettings {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(bevy_window::WindowSettings {
            add_primary_window: true,
            exit_on_all_closed: true,
            close_when_requested: true,
        })
        .insert_resource(bevy_winit::WinitSettings::game())
        .insert_resource(bevy_asset::AssetServerSettings {
            asset_folder: "res".to_string(),
            watch_for_changes: false,
        });
    }
}

pub struct FlatEngineCore;
impl PluginGroup for FlatEngineCore {
    fn build(&mut self, group: &mut bevy_app::PluginGroupBuilder) {
        group.add(FlatCorePlugin);

        // bevy_render::RenderStage::Extract;
        // bevy_sprite::Anchor::BottomCenter;
    }
}

pub struct FlatCorePlugin;
impl Plugin for FlatCorePlugin {
    fn build(&self, _app: &mut bevy_app::App) {}
}
