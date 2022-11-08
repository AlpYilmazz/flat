// use lazy_static::lazy_static;

use crate::{
    render::{
        camera::CameraUniform,
        resource::{
            buffer::{MeshVertex, Vertex},
            pipeline::{RenderPipeline, RenderPipelineBuilder, RenderPipelineDescriptor},
            shader::Shader,
            uniform::UniformDesc,
        },
        texture::TextureDesc,
    },
    transform::ModelMatrix,
    util::EngineDefault,
};

// lazy_static! {
//     static ref TEST_WGSL_SHADER_TARGETS: ShaderTargets = ShaderTargets {
//         vertex_buffers: vec![Vertex::layout()],
//         fragment_targets: vec![Some(wgpu::ColorTargetState {
//             format: wgpu::TextureFormat::engine_default(),
//             blend: Some(wgpu::BlendState::REPLACE),
//             write_mask: wgpu::ColorWrites::ALL,
//         })],
//     };
// }

pub trait ShaderInstance {
    fn pipeline(device: &wgpu::Device) -> RenderPipeline;
}

pub struct TestWgsl;
impl ShaderInstance for TestWgsl {
    fn pipeline(device: &wgpu::Device) -> RenderPipeline {
        RenderPipelineBuilder::new(device)
            .with_bind(UniformDesc::<CameraUniform>::default())
            .with_bind(UniformDesc::<ModelMatrix>::default())
            .with_bind(TextureDesc::default())
            .create_usual(&RenderPipelineDescriptor {
                shader: &Shader::from_with(
                    device.create_shader_module(wgpu::include_wgsl!("../res/test.wgsl")),
                    vec![Vertex::layout()],
                    vec![Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::engine_default(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                ),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                depth_stencil: true,
            })
    }
}
