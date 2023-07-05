use bevy::{
    asset::HandleId,
    ecs::system::SystemState,
    prelude::{Deref, DerefMut, FromWorld, Res, ResMut, Resource, World},
    utils::HashMap,
};
use encase::ShaderType;

use crate::{
    core_pipeline::bind::PipelineCommons,
    render::{
        camera::component::CameraUniforms,
        resource::{
            buffer::{MeshVertex, VertexC, Vertex, VertexBase},
            component_uniform::{ComponentUniforms, ModelUniform},
            pipeline::{
                BindGroupLayout, FragmentState, PipelineCache, PipelineLayoutDescriptor,
                RenderPipelineDescriptor, RenderPipelineId, VertexState,
            },
            renderer::{RenderDevice, RenderQueue},
            shader::Shader,
        },
        texture::{self, GpuTexture, Image, PixelFormat, RawImage},
        RenderAssets, uniform::{RadiusUniform, ColorUniform},
    },
    util::EngineDefault,
};

use super::{SPRITE_SHADER_HANDLE, CIRCLE_SHADER_HANDLE, TRIANGLE_SHADER_HANDLE};

#[derive(Resource)]
pub struct SpritePipeline {
    pub pipeline_id: RenderPipelineId,
    // PipelineCommons: model, view
    pub texture_layout: BindGroupLayout,
    pub dummy_texture: GpuTexture,
    pub dummy_texture_bind_group: wgpu::BindGroup,
}

impl FromWorld for SpritePipeline {
    fn from_world(world: &mut World) -> Self {
        let mut state: SystemState<(
            Res<RenderDevice>,
            Res<RenderQueue>,
            ResMut<PipelineCache>,
            Res<PipelineCommons>,
        )> = SystemState::new(world);
        let (render_device, render_queue, mut pipeline_cache, pipeline_commons) =
            state.get_mut(world);

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
                bind_group_layouts: vec![
                    pipeline_commons.model_layout.clone(),
                    pipeline_commons.view_layout.clone(),
                    texture_layout.clone(),
                ],
                push_constant_ranges: Vec::new(),
            },
            vertex: VertexState {
                shader: SPRITE_SHADER_HANDLE.typed(),
                entry_point: Shader::VS_ENTRY_DEFAULT,
                buffers: vec![VertexC::layout()],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DepthTexture::DEPTH_FORMAT, // wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        SpritePipeline {
            pipeline_id,
            texture_layout,
            dummy_texture,
            dummy_texture_bind_group,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TextureBindGroups(pub HashMap<HandleId, wgpu::BindGroup>);

pub fn create_texture_bind_groups(
    render_device: Res<RenderDevice>,
    sprite_pipeline: Res<SpritePipeline>,
    mut texture_bind_groups: ResMut<TextureBindGroups>,
    render_images: Res<RenderAssets<Image>>,
) {
    for (handle_id, gpu_image) in render_images.iter() {
        texture_bind_groups.entry(*handle_id).or_insert_with(|| {
            render_device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &sprite_pipeline.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        // resource: wgpu::BindingResource::TextureViewArray(),
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

#[derive(Resource)]
pub struct CirclePipeline {
    pub pipeline_id: RenderPipelineId,
    // PipelineCommons: model, view
    pub radius_layout: BindGroupLayout,
    pub color_layout: BindGroupLayout,
}

impl FromWorld for CirclePipeline {
    fn from_world(world: &mut World) -> Self {
        let mut state: SystemState<(
            Res<RenderDevice>,
            Res<RenderQueue>,
            ResMut<PipelineCache>,
            Res<PipelineCommons>,
        )> = SystemState::new(world);
        let (render_device, render_queue, mut pipeline_cache, pipeline_commons) =
            state.get_mut(world);

        let radius_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        // min_binding_size: None,
                        min_binding_size: Some(RadiusUniform::min_size()),
                    },
                    count: None,
                }],
                label: Some("circle_radius_layout"),
            });

        let color_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        // min_binding_size: None,
                        min_binding_size: Some(ColorUniform::min_size()),
                    },
                    count: None,
                }],
                label: Some("circle_color_layout"),
            });

        let pipeline_id = pipeline_cache.queue(RenderPipelineDescriptor {
            label: None,
            layout: PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: vec![
                    pipeline_commons.model_layout.clone(),
                    pipeline_commons.view_layout.clone(),
                    radius_layout.clone(),
                    color_layout.clone(),
                ],
                push_constant_ranges: Vec::new(),
            },
            vertex: VertexState {
                shader: CIRCLE_SHADER_HANDLE.typed(),
                entry_point: Shader::VS_ENTRY_DEFAULT,
                buffers: vec![Vertex::layout()],
            },
            fragment: Some(FragmentState {
                shader: CIRCLE_SHADER_HANDLE.typed(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DepthTexture::DEPTH_FORMAT, // wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        CirclePipeline {
            pipeline_id,
            radius_layout,
            color_layout,
        }
    }
}

#[derive(Default, Resource)]
pub struct CircleBindGroups {
    pub radius_bind_group: Option<wgpu::BindGroup>,
    pub color_bind_group: Option<wgpu::BindGroup>,
}

pub fn create_circle_bind_groups(
    circle_pipeline: Res<CirclePipeline>,
    mut circle_bind_groups: ResMut<CircleBindGroups>,
    render_device: Res<RenderDevice>,
    radius_uniforms: Res<ComponentUniforms<RadiusUniform>>,
    color_uniforms: Res<ComponentUniforms<ColorUniform>>,
) {
    let Some(radius_binding) = radius_uniforms.binding() else {
        return;
    };
    let radius_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &circle_pipeline.radius_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: radius_binding,
        }],
    });

    let Some(color_binding) = color_uniforms.binding() else {
        return;
    };
    let color_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &circle_pipeline.color_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: color_binding,
        }],
    });

    circle_bind_groups.radius_bind_group = Some(radius_bind_group);
    circle_bind_groups.color_bind_group = Some(color_bind_group);
}

#[derive(Resource)]
pub struct TrianglePipeline {
    pub pipeline_id: RenderPipelineId,
    // PipelineCommons: model, view
    pub color_layout: BindGroupLayout,
}

impl FromWorld for TrianglePipeline {
    fn from_world(world: &mut World) -> Self {
        let mut state: SystemState<(
            Res<RenderDevice>,
            ResMut<PipelineCache>,
            Res<PipelineCommons>,
        )> = SystemState::new(world);
        let (render_device, mut pipeline_cache, pipeline_commons) =
            state.get_mut(world);

        let color_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        // min_binding_size: None,
                        min_binding_size: Some(ColorUniform::min_size()),
                    },
                    count: None,
                }],
                label: Some("triangle_color_layout"),
            });

        let pipeline_id = pipeline_cache.queue(RenderPipelineDescriptor {
            label: None,
            layout: PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: vec![
                    pipeline_commons.model_layout.clone(),
                    pipeline_commons.view_layout.clone(),
                    color_layout.clone(),
                ],
                push_constant_ranges: Vec::new(),
            },
            vertex: VertexState {
                shader: TRIANGLE_SHADER_HANDLE.typed(),
                entry_point: Shader::VS_ENTRY_DEFAULT,
                buffers: vec![VertexBase::layout()],
            },
            fragment: Some(FragmentState {
                shader: TRIANGLE_SHADER_HANDLE.typed(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DepthTexture::DEPTH_FORMAT, // wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        TrianglePipeline {
            pipeline_id,
            color_layout,
        }
    }
}

#[derive(Default, Resource)]
pub struct TriangleBindGroups {
    // TODO: fix double create with circle pipeline
    pub color_bind_group: Option<wgpu::BindGroup>,
}

pub fn create_triangle_bind_groups(
    triangle_pipeline: Res<TrianglePipeline>,
    mut triangle_bind_groups: ResMut<TriangleBindGroups>,
    render_device: Res<RenderDevice>,
    color_uniforms: Res<ComponentUniforms<ColorUniform>>,
) {
    let Some(color_binding) = color_uniforms.binding() else {
        return;
    };
    let color_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &triangle_pipeline.color_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: color_binding,
        }],
    });

    triangle_bind_groups.color_bind_group = Some(color_bind_group);
}