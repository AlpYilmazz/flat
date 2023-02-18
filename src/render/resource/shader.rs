use bevy::{reflect::TypeUuid, asset::{AssetLoader, LoadedAsset}};

use crate::render::RenderDevice;

#[derive(TypeUuid)]
#[uuid = "4B8302DA-21AD-401F-AF45-1DFD956B80B5"]
pub struct Shader {
    pub raw: String,
}

impl Shader {
    pub const VS_ENTRY_DEFAULT: &'static str = "vs_main";
    pub const FS_ENTRY_DEFAULT: &'static str = "fs_main";

    pub fn from_wgsl(source: &str) -> Self {
        Self {
            raw: source.to_string(),
        }
    }

    pub fn compile(&self, render_device: &RenderDevice) -> wgpu::ShaderModule {
        render_device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.raw.as_str().into()),
        })
    }
}

#[derive(Default)]
pub struct ShaderLoader;
impl AssetLoader for ShaderLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(
                Shader {
                    raw: String::from_utf8(bytes.to_owned()).unwrap()
                }
            ));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wgsl"]
    }
}
