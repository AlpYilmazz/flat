use bevy::{
    asset::HandleId,
    ecs::system::SystemState,
    prelude::{FromWorld, Res, ResMut, Resource, World},
    utils::HashMap,
};
use encase::ShaderType;

use crate::{render::{
    resource::{pipeline::{BindGroupLayout, PipelineCache, RenderPipelineDescriptor, PipelineLayoutDescriptor, VertexState, FragmentState, RenderPipelineId}, shader::Shader, buffer::{Vertex, MeshVertex}},
    texture::{GpuTexture, Image, PixelFormat, RawImage},
    RenderAssets, RenderDevice, RenderQueue,
}, util::EngineDefault};

use super::SPRITE_SHADER_HANDLE;

#[derive(Resource)]
pub struct SpritePipeline {
    pub pipeline_id: RenderPipelineId,
    pub model_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
    pub dummy_texture: GpuTexture,
    pub dummy_texture_bind_group: wgpu::BindGroup,
}

impl FromWorld for SpritePipeline {
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
                        min_binding_size: None,
                        // min_binding_size: Some(bevy::render::view::ViewUniform::min_size()),
                    },
                    count: None,
                }],
                label: Some("sprite_model_layout"),
            });

        let view_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(bevy::render::view::ViewUniform::min_size()),
                    },
                    count: None,
                }],
                label: Some("sprite_view_layout"),
            });

        let texture_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite_texture_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
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

        let dummy_texture_bind_group =
            render_device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&dummy_texture.view),
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
                bind_group_layouts: vec![model_layout.clone(), view_layout.clone(), texture_layout.clone()],
                push_constant_ranges: Vec::new(),
            },
            vertex: VertexState {
                shader: SPRITE_SHADER_HANDLE.typed(),
                entry_point: Shader::VS_ENTRY_DEFAULT,
                buffers: vec![Vertex::layout()],
            },
            fragment: Some(FragmentState {
                shader: SPRITE_SHADER_HANDLE.typed(),
                entry_point: Shader::FS_ENTRY_DEFAULT,
                targets: vec![Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::engine_default(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        SpritePipeline {
            pipeline_id,
            model_layout,
            view_layout,
            texture_layout,
            dummy_texture,
            dummy_texture_bind_group,
        }
    }
}

#[derive(Resource, Default)]
pub struct TextureBindGroups(pub HashMap<HandleId, wgpu::BindGroup>);

pub fn create_texture_bind_groups(
    device: Res<RenderDevice>,
    sprite_shader: Res<SpritePipeline>,
    mut texture_bind_groups: ResMut<TextureBindGroups>,
    render_images: Res<RenderAssets<Image>>,
) {
    for (handle_id, gpu_image) in render_images.iter() {
        texture_bind_groups.0.entry(*handle_id).or_insert_with(|| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &sprite_shader.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&gpu_image.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&gpu_image.sampler),
                    },
                ],
            })
        });
    }
}
