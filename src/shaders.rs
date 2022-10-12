// use lazy_static::lazy_static;

use crate::{
    render::{
        camera::Camera,
        resource::{
            bind::ManyBindingSet,
            buffer::{MeshVertex, Vertex},
            pipeline::RenderPipeline,
            shader::Shader,
            uniform::Uniform,
        },
    },
    transform::GlobalTransform,
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

pub trait ShaderInstance<'a, const N: usize> {
    type BindingSets: ManyBindingSet<N>;

    fn pipeline(device: &wgpu::Device, binds: Self::BindingSets) -> RenderPipeline;
    fn layouts(device: &wgpu::Device, binds: Self::BindingSets) -> [wgpu::BindGroupLayout; N] {
        binds.into_layouts(device)
    }
    fn bind_groups(device: &wgpu::Device, binds: Self::BindingSets) -> [wgpu::BindGroup; N] {
        binds.into_bind_groups(device)
    }
}

pub struct TestWgsl;
impl<'a> ShaderInstance<'a, 1> for TestWgsl {
    type BindingSets = ((&'a Uniform<Camera>, &'a Uniform<GlobalTransform>),);

    fn pipeline(device: &wgpu::Device, binds: Self::BindingSets) -> RenderPipeline {
        let layouts = Self::layouts(device, binds);
        RenderPipeline::create_usual(
            &device,
            &[&layouts[0]],
            &Shader::from_with(
                device.create_shader_module(wgpu::include_wgsl!("../res/test.wgsl")),
                vec![Vertex::layout()],
                vec![Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::engine_default(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            ),
            wgpu::PrimitiveTopology::TriangleList,
            false,
        )
    }
}
