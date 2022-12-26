use bevy::{
    asset::HandleId,
    ecs::system::SystemState,
    prelude::{
        Color, Commands, Component, Entity, FromWorld, GlobalTransform, Handle, HandleUntyped,
        Image, Mat4, Query, Res, ResMut, Resource, Shader, Vec2, World,
    },
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            encase::private::WriteInto, BindGroup, BindGroupLayout, DynamicUniformBuffer,
            FragmentState, RenderPipelineDescriptor, ShaderType, SpecializedRenderPipeline,
            VertexBufferLayout, VertexState,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, DefaultImageSampler, GpuImage, ImageSampler, TextureFormatPixelInfo},
        Extract,
    },
    utils::HashMap,
};

use super::{ExtractedSprite, ExtractedSprites, Sprite, SPRITE_SHADER_HANDLE};

#[derive(Resource)]
pub struct SpritePipeline {
    pub model_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
    pub dummy_texture: GpuImage,
    pub dummy_texture_bind_group: BindGroup,
}

impl FromWorld for SpritePipeline {
    fn from_world(world: &mut World) -> Self {
        let mut state: SystemState<(
            Res<RenderDevice>,
            Res<RenderQueue>,
            Res<DefaultImageSampler>,
        )> = SystemState::new(world);
        let (render_device, render_queue, default_sampler) = state.get(world);

        let model_layout = render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let view_layout = render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let texture_layout = render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            let image = Image::new_fill(
                wgpu::Extent3d::default(),
                wgpu::TextureDimension::D2,
                &[255u8; 4],
                wgpu::TextureFormat::bevy_default(),
            );
            let texture = render_device.create_texture(&image.texture_descriptor);
            let sampler = match image.sampler_descriptor {
                ImageSampler::Default => (**default_sampler).clone(),
                ImageSampler::Descriptor(descriptor) => render_device.create_sampler(&descriptor),
            };

            let format_size = image.texture_descriptor.format.pixel_size();
            render_queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &image.data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(
                            image.texture_descriptor.size.width * format_size as u32,
                        )
                        .unwrap(),
                    ),
                    rows_per_image: None,
                },
                image.texture_descriptor.size,
            );
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            GpuImage {
                texture,
                texture_view,
                texture_format: image.texture_descriptor.format,
                sampler,
                size: Vec2::new(
                    image.texture_descriptor.size.width as f32,
                    image.texture_descriptor.size.height as f32,
                ),
            }
        };

        let dummy_texture_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_texture.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dummy_texture.sampler),
                },
            ],
        });

        SpritePipeline {
            model_layout,
            view_layout,
            texture_layout,
            dummy_texture,
            dummy_texture_bind_group,
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct SpritePipelineKey: u32 {
        const NONE                        = 0;
        const COLORED                     = (1 << 0);
    }
}

impl SpritePipelineKey {
    pub fn is_colored(&self) -> bool {
        self.contains(Self::COLORED)
    }
}

impl SpecializedRenderPipeline for SpritePipeline {
    type Key = SpritePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();
        let mut vertex_formats = vec![wgpu::VertexFormat::Float32x3, wgpu::VertexFormat::Float32x2];

        if key.is_colored() {
            shader_defs.push("COLORED".to_string());
            vertex_formats.push(wgpu::VertexFormat::Float32x4);
        }

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(wgpu::VertexStepMode::Vertex, vertex_formats);

        let format = wgpu::TextureFormat::bevy_default();

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: SPRITE_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vs_main".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: SPRITE_SHADER_HANDLE.typed::<Shader>(),
                shader_defs,
                entry_point: "fs_main".into(),
                targets: vec![Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            layout: Some(vec![
                self.model_layout.clone(),
                self.view_layout.clone(),
                self.texture_layout.clone(),
            ]),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
            label: Some("sprite_pipeline".into()),
        }
    }
}

#[derive(Resource, Default)]
pub struct TextureBindGroups(pub HashMap<HandleId, BindGroup>);

pub fn create_texture_bind_groups(
    device: Res<RenderDevice>,
    sprite_shader: Res<SpritePipeline>,
    mut texture_bind_groups: ResMut<TextureBindGroups>,
    render_images: Res<RenderAssets<Image>>,
) {
    for (handle, gpu_image) in render_images.iter() {
        texture_bind_groups.0.entry(handle.id()).or_insert_with(|| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &sprite_shader.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&gpu_image.texture_view),
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

#[derive(Clone, ShaderType)]
pub struct ModelUniform {
    model: Mat4,
}

#[derive(Resource, Default)]
pub struct ModelUniforms {
    pub uniforms: DynamicUniformBuffer<ModelUniform>,
}

#[derive(Component)]
pub struct ModelUniformOffset {
    pub offset: u32,
}

pub fn prepare_model_uniforms(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut model_uniforms: ResMut<ModelUniforms>,
    extracted_sprites: Res<ExtractedSprites>,
) {
    model_uniforms.uniforms.clear();
    for ExtractedSprite {
        entity, transform, ..
    } in &extracted_sprites.sprites
    {
        let model_uniform_offset = ModelUniformOffset {
            offset: model_uniforms.uniforms.push(ModelUniform {
                model: transform.compute_matrix(),
            }),
        };

        commands.get_or_spawn(*entity).insert(model_uniform_offset);
    }

    model_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

pub trait UniformHandle {
    type Uniform: ShaderType + WriteInto + Clone;

    fn create_uniform(&self) -> Self::Uniform;
}

pub fn extract_sprites(
    mut extracted_sprites: ResMut<ExtractedSprites>,
    sprites: Extract<Query<(Entity, &Sprite, &GlobalTransform, &Handle<Image>)>>,
) {
    for (entity, sprite, transform, texture) in sprites.iter() {
        extracted_sprites.sprites.push(ExtractedSprite {
            entity,
            transform: transform.clone(),
            image_handle: texture.id(),
            color: sprite.color,
        })
    }
}
