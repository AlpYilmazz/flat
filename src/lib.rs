use asset::FlatAssetPlugin;
use bevy_app::{CoreStage, Plugin, PluginGroup};
use input::FlatInputPlugin;
use render::FlatRenderPlugin;
use transform::FlatTransformPlugin;
use window::{ExitOnWindowClose, FlatWindowPlugin, FlatWinitPlugin};

// pub mod legacy;
pub mod misc;
pub mod text;
pub mod texture;
pub mod util;

pub mod asset;
pub mod input;
pub mod hierarchy;
pub mod render;
pub mod shaders;
pub mod transform;
pub mod window;

/*
TypeUuid

6948DF80-14BD-4E04-8842-7668D9C001F5 - Text
4B8302DA-21AD-401F-AF45-1DFD956B80B5 - ShaderSource
8628FE7C-A4E9-4056-91BD-FD6AA7817E39 - Mesh<V: MeshVertex>
ED280816-E404-444A-A2D9-FFD2D171F928 - BatchMesh<V: MeshVertex>
D952EB9F-7AD2-4B1B-B3CE-386735205990
3F897E85-62CE-4B2C-A957-FCF0CCE649FD
8E7C2F0A-6BB8-485C-917E-6B605A0DDF29
1AD2F3EF-87C8-46B4-BD1D-94C174C278EE
AA97B177-9383-4934-8543-0F91A7A02836
10929DF8-15C5-472B-9398-7158AB89A0A6 - Vertex: MeshVertex
*/

pub struct FlatEngineComplete;

impl PluginGroup for FlatEngineComplete {
    fn build(&mut self, group: &mut bevy_app::PluginGroupBuilder) {
        FlatEngineCore.build(group);
    }
}

pub struct FlatEngineCore;

impl PluginGroup for FlatEngineCore {
    fn build(&mut self, group: &mut bevy_app::PluginGroupBuilder) {
        group
            .add(FlatCorePlugin)
            .add(FlatTransformPlugin)
            .add(FlatInputPlugin)
            .add(FlatAssetPlugin)
            .add(FlatWindowPlugin)
            .add(FlatWinitPlugin {
                exit_on_close: ExitOnWindowClose::Primary,
                ..Default::default()
            })
            .add(FlatRenderPlugin);
    }
}

pub struct FlatCorePlugin;
impl Plugin for FlatCorePlugin {
    fn build(&self, _app: &mut bevy_app::App) {}
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
