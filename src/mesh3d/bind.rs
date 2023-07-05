use bevy::{
    ecs::system::SystemState,
    prelude::{FromWorld, Res, ResMut, Resource, World, Component, Deref, DerefMut}, utils::HashMap, asset::HandleId,
};
use encase::ShaderType;

use crate::{
    render::{
        camera::component::CameraUniforms,
        resource::{
            buffer::{MeshVertex, VertexTex3},
            component_uniform::{ComponentUniforms, ModelUniform},
            pipeline::{
                BindGroupLayout, FragmentState, PipelineCache, PipelineLayoutDescriptor,
                RenderPipelineDescriptor, VertexState,
            },
            renderer::{RenderDevice, RenderQueue},
            shader::Shader,
            specialized_pipeline::{PipelineSpecialize, Specialized},
        },
        texture::{GpuTexture, ImageDim, PixelFormat, texture_arr::ImageArray, self}, RenderAssets,
    },
    util::EngineDefault,
};

use super::MESH_SHADER_HANDLE;

#[derive(Resource)]
pub struct MeshPipeline {
    pub model_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
    // pub texture_arr_layout: BindGroupLayout,
    pub dummy_texture_arr: GpuTexture,
    pub dummy_texture_arr_bind_group: wgpu::BindGroup,
}

impl FromWorld for MeshPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut state: SystemState<(
            Res<RenderDevice>,
            Res<RenderQueue>,
            ResMut<PipelineCache>,
            ResMut<Specialized<Self>>,
        )> = SystemState::new(world);
        let (render_device, render_queue, mut pipeline_cache, mut specialized_self) =
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
                label: Some("mesh_model_layout"),
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
                label: Some("mesh_view_layout"),
            });

        let dummy_texture_arr_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("dummy_texture_arr_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: std::num::NonZeroU32::new(6), // TODO
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let dummy_texture_arr = GpuTexture::create_texture_array(
            &render_device,
            &render_queue,
            &[255u8; 4*6], // TODO
            ImageDim {
                width: 1,
                heigth: 1,
                pixel: PixelFormat::RGBA8,
            },
            6, // TODO
        )
        .unwrap();

        let dummy_texture_arr_bind_group =
            render_device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &dummy_texture_arr_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&dummy_texture_arr.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&dummy_texture_arr.sampler),
                    },
                ],
            });

        let mesh_pipeline = MeshPipeline {
            model_layout,
            view_layout,
            // arr_texture_layout,
            dummy_texture_arr,
            dummy_texture_arr_bind_group,
        };

        const MESH_PIPELINE_KEYS: &'static [MeshPipelineKey] =
            &[MeshPipelineKey { texture_count: 6 }];

        for key in MESH_PIPELINE_KEYS {
            let id = pipeline_cache.queue(mesh_pipeline.specialize(&render_device, *key));
            specialized_self.pipelines.insert(*key, id);
        }

        mesh_pipeline
    }
}

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq)]
pub struct MeshPipelineKey {
    pub texture_count: u32,
}

impl PipelineSpecialize for MeshPipeline {
    type Key = MeshPipelineKey;

    fn specialize(&self, render_device: &RenderDevice, key: Self::Key) -> RenderPipelineDescriptor {
        let texture_arr_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mesh_texture_arr_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: std::num::NonZeroU32::new(key.texture_count),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        RenderPipelineDescriptor {
            label: None,
            layout: PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: vec![
                    self.model_layout.clone(),
                    self.view_layout.clone(),
                    texture_arr_layout.clone(),
                ],
                push_constant_ranges: Vec::new(),
            },
            vertex: VertexState {
                shader: MESH_SHADER_HANDLE.typed(),
                entry_point: Shader::VS_ENTRY_DEFAULT,
                buffers: vec![VertexTex3::layout()],
            },
            fragment: Some(FragmentState {
                shader: MESH_SHADER_HANDLE.typed(),
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
        }
    }
}

#[derive(Default, Resource)]
pub struct MeshBindGroups {
    pub model_bind_group: Option<wgpu::BindGroup>,
    pub view_bind_group: Option<wgpu::BindGroup>,
}

pub fn create_mesh3d_bind_groups(
    render_device: Res<RenderDevice>,
    mut mesh3d_bind_groups: ResMut<MeshBindGroups>,
    mesh3d_pipeline: Res<MeshPipeline>,
    model_uniforms: Res<ComponentUniforms<ModelUniform>>,
    view_uniforms: Res<ComponentUniforms<CameraUniforms>>,
) {
    let Some(model_binding) = model_uniforms.binding() else {
        return;
    };
    let model_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &mesh3d_pipeline.model_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: model_binding,
        }],
    });

    let Some(view_binding) = view_uniforms.binding() else {
        return;
    };
    let view_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &mesh3d_pipeline.view_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: view_binding,
        }],
    });

    mesh3d_bind_groups.model_bind_group = Some(model_bind_group);
    mesh3d_bind_groups.view_bind_group = Some(view_bind_group);
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TextureArrayBindGroups(pub HashMap<HandleId, wgpu::BindGroup>);

pub fn create_texture_arr_bind_groups(
    render_device: Res<RenderDevice>,
    // mesh_pipeline: Res<MeshPipeline>,
    mut texture_arr_bind_groups: ResMut<TextureArrayBindGroups>,
    render_images: Res<RenderAssets<ImageArray>>,
) {
    let texture_arr_layout =
            render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mesh_texture_arr_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: std::num::NonZeroU32::new(6), // TODO
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

    for (handle_id, gpu_image) in render_images.iter() {
        texture_arr_bind_groups.entry(*handle_id).or_insert_with(|| {
            render_device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &texture_arr_layout,
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