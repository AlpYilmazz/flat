use std::collections::HashMap;

use bevy_asset::{AssetEvent, AssetLoader, AssetServer, Assets, Handle, HandleId, LoadedAsset};
use bevy_ecs::{
    prelude::EventReader,
    system::{Res, ResMut},
};
use bevy_reflect::TypeUuid;

use crate::util::{AssetStore};

use super::buffer::{InstanceRaw, InstanceUnit, MeshVertex, Vertex};

pub struct ShaderTargets {
    pub vertex_buffers: Vec<wgpu::VertexBufferLayout<'static>>, // TODO: lifetime again
    pub fragment_targets: Vec<Option<wgpu::ColorTargetState>>,
}

impl Default for ShaderTargets {
    fn default() -> Self {
        Self {
            vertex_buffers: Default::default(),
            fragment_targets: Default::default(),
        }
    }
}

pub struct Shader {
    pub module: wgpu::ShaderModule,
    pub targets: ShaderTargets,
}

impl Shader {
    pub const VERTEX_ENTRY_POINT: &'static str = "vs_main";
    pub const FRAGMENT_ENTRY_POINT: &'static str = "fs_main";

    pub fn with(module: wgpu::ShaderModule) -> Self {
        Self {
            module,
            targets: Default::default(),
        }
    }

    pub fn with_final(
        module: wgpu::ShaderModule,
        vertex_buffers: Vec<wgpu::VertexBufferLayout<'static>>, // TODO: lifetime
        fragment_targets: Vec<Option<wgpu::ColorTargetState>>,
    ) -> Self {
        Self {
            module,
            targets: ShaderTargets {
                vertex_buffers,
                fragment_targets,
            },
        }
    }

    pub fn with_targets(module: wgpu::ShaderModule, targets: ShaderTargets) -> Self {
        Self { module, targets }
    }

    pub fn add_vertex<V: MeshVertex>(&mut self) {
        self.targets.vertex_buffers.push(V::layout());
    }

    pub fn add_fragment_target(&mut self, target: wgpu::ColorTargetState) {
        self.targets.fragment_targets.push(Some(target));
    }
}

#[derive(Default)]
pub struct Shaders(
    pub HashMap<HandleId, Shader>,
    pub HashMap<HandleId, ShaderTargets>,
);

#[derive(TypeUuid)]
#[uuid = "4B8302DA-21AD-401F-AF45-1DFD956B80B5"]
pub struct ShaderSource(String);

impl ShaderSource {
    pub fn compile(self, device: &wgpu::Device) -> Shader {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(self.0)),
        });
        Shader::with(module)
    }

    pub fn compile_with_targets(self, device: &wgpu::Device, targets: ShaderTargets) -> Shader {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(self.0)),
        });
        Shader::with_targets(module, targets)
    }
}

pub struct ShaderSourceLoader;
impl AssetLoader for ShaderSourceLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy_asset::LoadContext,
    ) -> bevy_asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(ShaderSource(
                String::from_utf8(bytes.to_owned()).unwrap(),
            )));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wgsl"]
    }
}

pub fn compile_shaders(
    device: Res<wgpu::Device>,
    mut events: EventReader<AssetEvent<ShaderSource>>,
    mut sources: ResMut<Assets<ShaderSource>>,
    // mut shaders: ResMut<Shaders>,
    mut shaders: ResMut<AssetStore<Shader>>,
    mut shader_targets: ResMut<AssetStore<ShaderTargets>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                let handle_id = handle.into();
                let shader_source = sources.remove(handle).unwrap();
                let shader = shader_source.compile_with_targets(
                    device.as_ref(),
                    shader_targets.remove(&handle_id).unwrap(),
                );
                shaders.insert(handle_id, shader);
            }
            _ => {}
        }
    }
}

pub fn load_test_shader(
    config: Res<wgpu::SurfaceConfiguration>,
    asset_server: Res<AssetServer>,
    mut shader_targets: ResMut<AssetStore<ShaderTargets>>,
) {
    let path = "res/basic.wgsl";
    let _shader_handle = load_shader(
        &asset_server,
        &mut shader_targets,
        path,
        ShaderTargets {
            vertex_buffers: vec![Vertex::layout(), InstanceRaw::layout()],
            fragment_targets: vec![Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        },
    );
    let _shader_handle_weak: Handle<ShaderSource> = Handle::weak(HandleId::from(path));
}

pub fn load_shader(
    asset_server: &AssetServer,
    shader_targets: &mut AssetStore<ShaderTargets>,
    path: &str,
    targets: ShaderTargets,
) -> Handle<ShaderSource> {
    let shader_handle: Handle<ShaderSource> = asset_server.load(path);
    shader_targets.insert(shader_handle.id, targets);

    shader_handle
}
