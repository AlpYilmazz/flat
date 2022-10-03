use asset::FlatAssetPlugin;
use bevy_app::{CoreStage, Plugin, PluginGroup};
use bevy_asset::{AssetLoader, AssetServer, FileAssetIo, LoadedAsset};
use bevy_ecs::{
    schedule::{StageLabel, SystemStage},
    system::{Commands, Res},
};
use bevy_reflect::TypeUuid;
use cgmath::*;
use input::FlatInputPlugin;
use render::{mesh::GpuMesh, resource::buffer::Vertex, FlatRenderPlugin};
use wgpu::{include_wgsl, util::DeviceExt};
use window::{FlatWinitPlugin, FlatWindowPlugin};
use winit::{event::*, window::Window};

// pub mod legacy;
pub mod camera;
pub mod render;
pub mod text;
pub mod texture;
pub mod util;

pub mod asset;
pub mod input;
pub mod window;

/*
TypeUuid

6948DF80-14BD-4E04-8842-7668D9C001F5 - Text
4B8302DA-21AD-401F-AF45-1DFD956B80B5 - ShaderSource
8628FE7C-A4E9-4056-91BD-FD6AA7817E39
10929DF8-15C5-472B-9398-7158AB89A0A6
ED280816-E404-444A-A2D9-FFD2D171F928
D952EB9F-7AD2-4B1B-B3CE-386735205990
3F897E85-62CE-4B2C-A957-FCF0CCE649FD
8E7C2F0A-6BB8-485C-917E-6B605A0DDF29
1AD2F3EF-87C8-46B4-BD1D-94C174C278EE
AA97B177-9383-4934-8543-0F91A7A02836
*/

#[derive(StageLabel)]
pub enum RenderStage {
    Render,
}

pub struct FlatEngineCore;
pub struct FlatEngineComplete;

impl PluginGroup for FlatEngineCore {
    fn build(&mut self, group: &mut bevy_app::PluginGroupBuilder) {
        group
            .add(FlatCorePlugin)
            .add(FlatInputPlugin)
            .add(FlatAssetPlugin)
            .add_after::<FlatAssetPlugin, FlatRenderPlugin>(FlatRenderPlugin)
            .add(FlatWindowPlugin)
            .add(FlatWinitPlugin::default());
    }
}

impl PluginGroup for FlatEngineComplete {
    fn build(&mut self, group: &mut bevy_app::PluginGroupBuilder) {
        let mut flat_engine_core = FlatEngineCore;
        flat_engine_core.build(group);
    }
}

pub struct FlatCorePlugin;
impl Plugin for FlatCorePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_stage_after(
            CoreStage::Last,
            RenderStage::Render,
            SystemStage::parallel(),
        );
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

pub fn create_wgpu_resources(window: Res<winit::window::Window>, mut commands: Commands) {
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window.as_ref()) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty() | wgpu::Features::TEXTURE_BINDING_ARRAY,
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
        },
        None, // trace_path
    ))
    .unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &config);

    commands.insert_resource(surface);
    commands.insert_resource(device);
    commands.insert_resource(queue);
    commands.insert_resource(config);
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    depth_texture: texture::Texture,
    asset_server: AssetServer,
    loaded: bool,
}

impl State {
    const VERTICES: &'static [Vertex] = &[
        Vertex {
            position: [-0.0868241, 0.49240386, 0.0],
            tex_coords: [0.4131759, 0.00759614],
        }, // A
        Vertex {
            position: [-0.49513406, 0.06958647, 0.0],
            tex_coords: [0.0048659444, 0.43041354],
        }, // B
        Vertex {
            position: [-0.21918549, -0.44939706, 0.0],
            tex_coords: [0.28081453, 0.949397],
        }, // C
        Vertex {
            position: [0.35966998, -0.3473291, 0.0],
            tex_coords: [0.85967, 0.84732914],
        }, // D
        Vertex {
            position: [0.44147372, 0.2347359, 0.0],
            tex_coords: [0.9414737, 0.2652641],
        }, // E
    ];

    const INDICES: &'static [u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

    const NUM_INSTANCES_PER_ROW: u32 = 1;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = Vector3::new(
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    const BACKGROUND_VERTICES: &'static [Vertex] = &[
        Vertex {
            position: [-0.5, 0.5, 0.0],
            tex_coords: [0.0, 0.0],
        }, // A
        Vertex {
            position: [-0.5, -0.5, 0.0],
            tex_coords: [0.0, 1.0],
        }, // B
        Vertex {
            position: [0.5, -0.5, 0.0],
            tex_coords: [1.0, 1.0],
        }, // C
        Vertex {
            position: [0.5, 0.5, 0.0],
            tex_coords: [1.0, 0.0],
        }, // E
    ];

    const BACKGROUND_INDICES: &'static [u16] = &[0, 1, 2, 0, 2, 3];

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty() | wgpu::Features::TEXTURE_BINDING_ARRAY,
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // trace_path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "Depth Texture");

        let asset_server = AssetServer::new(FileAssetIo::new(".", false));
        for file in ["posx", "negx", "posy", "negy", "posz", "negz"] {
            let path = format!("res/skybox/{file}.jpg");
            // asset_server.load_bytes(&path);
            // futures_lite::future::block_on(asset_server.load_bytes_async(path));
        }
        let loaded = false;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,

            asset_server,
            loaded,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size; // Copy
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
        // match self.asset_server.get_bytes() {
        //     Some(bytes) => {
        //         println!("{:?}", &bytes[1000..1020]);
        //         self.loaded = true;
        //     }
        //     None => {
        //         if !self.loaded {
        //             println!("Not loaded");
        //         }
        //     },
        // }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // for (pipeline_id, render_refs) in &self.render_refs {
            //     self.set_pipeline(&mut render_pass, *pipeline_id);
            //     for render_ref in render_refs {
            //         self.render_pass_draw(&mut render_pass, render_ref);
            //     }
            // }
        } // drop(render_pass) <- mut borrow encoder <- mut borrow self

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    fn set_pipeline<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        render_pipeline_id: usize,
    ) {
        // let render_pipeline =
        //     &self.render_resources.render_pipelines[render_pipeline_id];

        // render_pass.set_pipeline(render_pipeline);
    }

    fn render_pass_draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        // render_ref: &RenderRef,
    ) {
        // let mesh_0 =
        //     &self.render_resources.meshes[render_ref.mesh];

        // for (index, bind_group_id) in (&render_ref.bind_groups).into_iter().enumerate() {
        //     let bind_group = &self.render_resources.bind_groups[*bind_group_id];
        //     render_pass.set_bind_group(index as u32, bind_group, &[]);
        // }
        // render_pass.set_vertex_buffer(0, mesh_0.vertex_buffer.slice(..));
        // // TODO:
        // // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        // match &mesh_0.assembly {
        //     mesh::GpuMeshAssembly::Indexed {
        //         index_buffer,
        //         index_count,
        //         index_format
        //     } => {
        //         render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
        //         render_pass.draw_indexed(0..*index_count as u32, 0, 0..self.instances.len() as u32);
        //     },
        //     mesh::GpuMeshAssembly::NonIndexed {
        //         vertex_count
        //     } => {
        //         render_pass.draw(0..*vertex_count as u32, 0..self.instances.len() as u32);
        //     },
        // }
    }
}

// let pixel_size = std::mem::size_of::<[u8;4]>() as u32;
//         let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
//         let unpadded_bytes_per_row = pixel_size * self.size.width;
//         let padding = (align - unpadded_bytes_per_row % align) % align;
//         let padded_bytes_per_row = unpadded_bytes_per_row + padding;

//         // println!("{}\n{}\n{}\n", padded_bytes_per_row, self.size.height,
//         //     padded_bytes_per_row * self.size.height);

//         let frame = output.texture.as_image_copy();
//         encoder.copy_texture_to_buffer(
//             frame,
//             wgpu::ImageCopyBuffer {
//                 buffer: &self.framesave_buffer,
//                 layout: wgpu::ImageDataLayout {
//                     offset: 0,
//                     bytes_per_row: NonZeroU32::new(padded_bytes_per_row),
//                     rows_per_image: NonZeroU32::new(self.size.height),
//                 },
//             },
//             wgpu::Extent3d {
//                 width: self.size.width,
//                 height: self.size.height,
//                 depth_or_array_layers: 1,
//             },
//         );

//         let buffer_slice = self.framesave_buffer.slice(..);
//         let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
//         buffer_slice.map_async(
//             wgpu::MapMode::Read,
//             move |result| {
//                 tx.send(result).unwrap();
//             }
//         );
//         // wait for the GPU to finish
//         self.device.poll(wgpu::Maintain::Wait);

//         let result = pollster::block_on(rx.receive());

//         match result {
//             Some(Ok(())) => {
//                 let padded_data = buffer_slice.get_mapped_range();
//                 let data = padded_data
//                     .chunks(padded_bytes_per_row as _)
//                     .map(|chunk| &chunk[..unpadded_bytes_per_row as _])
//                     .flatten()
//                     .map(|x| *x)
//                     .collect::<Vec<_>>();
//                 drop(padded_data);
//                 self.framesave_buffer.unmap();
//                 self.recorded_frames.push(data);
//             }
//             _ => eprintln!("Something went wrong"),
//         }

fn save_gif(
    path: &str,
    frames: &mut Vec<Vec<u8>>,
    speed: i32,
    w: u16,
    h: u16,
) -> anyhow::Result<()> {
    use gif::{Encoder, Frame, Repeat};

    let mut image = std::fs::File::create(path)?;
    let mut encoder = Encoder::new(&mut image, w, h, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for mut frame in frames {
        encoder.write_frame(&Frame::from_rgba_speed(w, h, &mut frame, speed))?;
    }

    Ok(())
}
