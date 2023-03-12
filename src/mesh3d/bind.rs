use bevy::{prelude::{Resource, FromWorld, Res, World, ResMut}, ecs::system::SystemState};
use encase::ShaderType;

use crate::{render::{resource::{pipeline::{RenderPipelineId, BindGroupLayout, PipelineCache, RenderPipelineDescriptor, PipelineLayoutDescriptor, VertexState, FragmentState}, renderer::{RenderDevice, RenderQueue}, component_uniform::{ModelUniform, ComponentUniforms}, shader::Shader, buffer::{Vertex3DTex, MeshVertex}}, texture::{GpuTexture, RawImage, PixelFormat}, camera::component::CameraUniforms}, util::EngineDefault};

use super::MESH3D_3TEX_SHADER_HANDLE;


#[derive(Resource)]
pub struct Mesh3DPipeline<const ARR_TEX_N: usize> {
    pub pipeline_id: RenderPipelineId,
    pub model_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
    pub arr_texture_layout: BindGroupLayout,
    pub dummy_texture: GpuTexture,
    pub dummy_arr_texture_bind_group: wgpu::BindGroup,
}

impl<const ARR_TEX_N: usize> FromWorld for Mesh3DPipeline<ARR_TEX_N> {
    fn from_world(world: &mut World) -> Self {
        let mut state: SystemState<(Res<RenderDevice>, Res<RenderQueue>, ResMut<PipelineCache>)> =
            SystemState::new(world);
        let (render_device, render_queue, mut pipeline_cache) =
            state.get_mut(world);

        let model_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        // min_binding_size: None,
                        min_binding_size: Some(ModelUniform::min_size()),
                    },
                    count: None,
                }],
                label: Some("mesh3d_model_layout"),
            });

        let view_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(CameraUniforms::min_size()),
                    },
                    count: None,
                }],
                label: Some("mesh3d_view_layout"),
            });

        let arr_texture_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mesh3d_arr_texture_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: std::num::NonZeroU32::new(ARR_TEX_N as u32),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let dummy_texture = {
            let texture = GpuTexture::from_raw_image(
                &render_device,
                &render_queue,
                &RawImage::new(&[255u8; 4], (1, 1), PixelFormat::RGBA8),
                None,
            )
            .unwrap();
            texture
        };

        let dummy_texture_n: [&wgpu::TextureView; ARR_TEX_N] = std::array::from_fn(|i| &dummy_texture.view);

        let dummy_arr_texture_bind_group =
            render_device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &arr_texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&dummy_texture_n),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&dummy_texture.sampler),
                    },
                ],
            });

        let pipeline_id = pipeline_cache.queue(RenderPipelineDescriptor {
            label: None,
            layout: PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: vec![model_layout.clone(), view_layout.clone(), arr_texture_layout.clone()],
                push_constant_ranges: Vec::new(),
            },
            vertex: VertexState {
                shader: MESH3D_3TEX_SHADER_HANDLE.typed(),
                entry_point: Shader::VS_ENTRY_DEFAULT,
                buffers: vec![Vertex3DTex::layout()],
            },
            fragment: Some(FragmentState {
                shader: MESH3D_3TEX_SHADER_HANDLE.typed(),
                entry_point: Shader::FS_ENTRY_DEFAULT,
                targets: vec![Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::engine_default(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Mesh3DPipeline {
            pipeline_id,
            model_layout,
            view_layout,
            arr_texture_layout,
            dummy_texture,
            dummy_arr_texture_bind_group,
        }
    }
}

#[derive(Default, Resource)]
pub struct Mesh3DBindGroups {
    pub model_bind_group: Option<wgpu::BindGroup>,
    pub view_bind_group: Option<wgpu::BindGroup>,
}

pub fn create_mesh3d_bind_groups<const N: usize>(
    mut mesh3d_bind_groups: ResMut<Mesh3DBindGroups>,
    render_device: Res<RenderDevice>,
    mesh3d_pipeline: Res<Mesh3DPipeline<N>>,
    model_uniforms: Res<ComponentUniforms<ModelUniform>>,
    view_uniforms: Res<ComponentUniforms<CameraUniforms>>,
) {
    let Some(model_binding) = model_uniforms.binding() else {
        return;
    };
    let model_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &mesh3d_pipeline.model_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: model_binding,
            },
        ],
    });

    let Some(view_binding) = view_uniforms.binding() else {
        return;
    };
    let view_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &mesh3d_pipeline.view_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: view_binding,
            },
        ],
    });

    mesh3d_bind_groups.model_bind_group = Some(model_bind_group);
    mesh3d_bind_groups.view_bind_group = Some(view_bind_group);
}
