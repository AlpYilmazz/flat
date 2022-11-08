use std::{any::TypeId, collections::HashMap};

use crate::util::Label;

use super::{
    bind::{BindingSet, BindingSetDesc, IntoBindingSetDesc},
    shader,
};

#[derive(Clone)]
pub struct RenderPipelineDescriptor<'a> {
    pub shader: &'a shader::Shader,
    pub primitive_topology: wgpu::PrimitiveTopology,
    pub depth_stencil: bool,
}

pub struct RenderPipelineBuilder<'a> {
    device: &'a wgpu::Device,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    locations: HashMap<TypeId, HashMap<Option<Label>, usize>>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            bind_group_layouts: Vec::new(),
            locations: HashMap::new(),
        }
    }

    pub fn with_bind<T: IntoBindingSetDesc>(mut self, label: Option<Label>, desc: T) -> Self {
        let desc = desc.into_binding_set_desc();
        let type_id = TypeId::of::<T::SetDesc>();
        match label {
            Some(_) => {
                if !self.locations.contains_key(&type_id) {
                    self.locations.insert(type_id, HashMap::new());
                }
                let locations = self.locations.get_mut(&type_id).unwrap();
                matches!(locations.get(&None), None);
                locations.insert(label, self.bind_group_layouts.len());
            }
            None => {
                assert!(!self.locations.contains_key(&type_id));
                let mut locations = HashMap::new();
                locations.insert(label /* None */, self.bind_group_layouts.len());
            }
        }
        self.bind_group_layouts
            .push(desc.bind_group_layout(self.device));
        self
    }

    pub fn create_usual(self, pipeline_desc: &RenderPipelineDescriptor) -> RenderPipeline {
        let device = self.device;
        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> =
            self.bind_group_layouts.iter().map(|e| e).collect();
        let RenderPipelineDescriptor {
            shader,
            primitive_topology,
            depth_stencil,
        } = pipeline_desc.clone();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: shader::Shader::VERTEX_ENTRY_POINT,
                buffers: &shader.targets.vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: shader::Shader::FRAGMENT_ENTRY_POINT,
                targets: &shader.targets.fragment_targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: primitive_topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires
                // Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: if depth_stencil {
                Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float, // texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(),     // 2.
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // let mut locations = Vec::with_capacity(self.bind_group_layouts.len());
        // for (type_id, mut loc_map) in self.locations.drain() {
        //     for (label, loc) in loc_map.drain() {
        //         locations.insert((type_id, label));
        //     }
        // }
        let locations = self.locations;

        RenderPipeline {
            locations,
            wgpu_pipeline: render_pipeline,
        }
    }
}

pub struct RenderPipeline {
    // locations: Vec<(TypeId, Option<Label>)>,
    locations: HashMap<TypeId, HashMap<Option<Label>, usize>>,
    pub wgpu_pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn bind_location<T: BindingSet>(&self, bind_label: &Option<Label>) -> Option<u32> {
        let bind_type_id = TypeId::of::<T::SetDesc>();
        self.locations
            .get(&bind_type_id)?
            .get(bind_label)
            .map(|loc| *loc as u32)
    }
}
