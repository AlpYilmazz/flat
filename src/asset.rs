use bevy_app::Plugin;
use bevy_asset::{AddAsset, AssetPlugin, AssetServerSettings};

use crate::{render::resource::shader::ShaderSource, Text, TextLoader};

pub struct FlatAssetPlugin;
impl Plugin for FlatAssetPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(AssetServerSettings {
            asset_folder: "res".to_string(),
            watch_for_changes: false,
        })
        .add_plugin(AssetPlugin)
        .add_asset_loader(TextLoader)
        .add_asset::<Text>()
        .add_asset::<ShaderSource>();
    }
}
