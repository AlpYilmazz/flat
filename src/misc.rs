use bevy::{
    asset::{AssetLoader, LoadedAsset},
    reflect::TypeUuid,
};

#[derive(TypeUuid)]
#[uuid = "6948DF80-14BD-4E04-8842-7668D9C001F5"]
pub struct Text(String);
pub struct TextLoader;
impl AssetLoader for TextLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
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