// use lazy_static::lazy_static;

use crate::{
    render::{
        camera::CameraUniform,
        resource::{
            bind::BindingSetDesc,
            buffer::{MeshVertex, Vertex},
            pipeline::RenderPipeline,
            shader::Shader,
            uniform::UniformDesc,
        },
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
    // type BindingSets: ManyBindingSet<N>;

    fn pipeline(device: &wgpu::Device) -> RenderPipeline;
    // fn layouts(device: &wgpu::Device, binds: Self::BindingSets) -> [wgpu::BindGroupLayout; N] {
    //     binds.into_layouts(device)
    // }
    // fn bind_groups(device: &wgpu::Device, binds: Self::BindingSets) -> [wgpu::BindGroup; N] {
    //     binds.into_bind_groups(device)
    // }
}

pub struct TestWgsl;
impl ShaderInstance for TestWgsl {
    // type BindingSets = (&'a Uniform<Camera>, &'a Uniform<GlobalTransform>);

    fn pipeline(device: &wgpu::Device) -> RenderPipeline {
        // let [camera_layout, model_layout] = Self::layouts(device, binds);

        let camera_layout = UniformDesc::<CameraUniform>::default()
            .as_ref()
            .bind_group_layout(device);
        let model_layout = UniformDesc::<ModelMatrix>::default()
            .as_ref()
            .bind_group_layout(device);

        RenderPipeline::create_usual(
            &device,
            &[&camera_layout, &model_layout],
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
