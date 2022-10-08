use bevy_app::Plugin;
use bevy_asset::{AddAsset, AssetPlugin, AssetServerSettings, AssetLoader, LoadedAsset};
use bevy_reflect::TypeUuid;

use crate::{render::resource::shader::ShaderSource};

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

#[derive(TypeUuid)]
#[uuid = "6948DF80-14BD-4E04-8842-7668D9C001F5"]
pub struct Text(String);
pub struct TextLoader;
impl AssetLoader for TextLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy_asset::LoadContext,
    ) -> bevy_asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(Text(
                String::from_utf8(bytes.to_owned()).unwrap(),
            )));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}
