use std::{num::NonZeroU32, sync::Arc, ops::Deref};

use bevy::{
    prelude::{Assets, Component, Handle, Res, ResMut, Resource},
    utils::HashMap,
};

use crate::render::RenderDevice;

use super::shader::Shader;

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderPipelineId(usize);

#[derive(Resource, Default)]
pub struct PipelineCache {
    id_to_ind: HashMap<RenderPipelineId, usize>,
    pipelines: Vec<wgpu::RenderPipeline>,
    waiting: Vec<(RenderPipelineId, RenderPipelineDescriptor)>,
}

impl PipelineCache {
    pub fn queue(&mut self, desc: RenderPipelineDescriptor) -> RenderPipelineId {
        let id = RenderPipelineId(self.pipelines.len() + self.waiting.len());
        self.waiting.push((id, desc));
        id
    }

    pub fn get(&self, id: &RenderPipelineId) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(*self.id_to_ind.get(&id)?)
    }

    fn create(
        &mut self,
        render_device: &RenderDevice,
        id: RenderPipelineId,
        desc: &RenderPipelineDescriptor,
        vs_module: &wgpu::ShaderModule,
        fs_module: Option<&wgpu::ShaderModule>,
    ) {
        let pipeline_layout =
            render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: desc.layout.label,
                bind_group_layouts: &desc
                    .layout
                    .bind_group_layouts
                    .iter()
                    .map(|b| b.value.as_ref())
                    .collect::<Vec<_>>(),
                push_constant_ranges: &desc.layout.push_constant_ranges,
            });

        let pipeline = render_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: desc.label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: vs_module,
                entry_point: desc.vertex.entry_point,
                buffers: &desc.vertex.buffers,
            },
            primitive: desc.primitive,
            depth_stencil: desc.depth_stencil.clone(),
            multisample: desc.multisample,
            fragment: match fs_module {
                Some(fs_module) => Some(wgpu::FragmentState {
                    module: fs_module,
                    entry_point: desc.fragment.as_ref().unwrap().entry_point, // SAFE unwrap, bound to fs_module, inside map
                    targets: &desc.fragment.as_ref().unwrap().targets, // SAFE unwrap, bound to fs_module, inside map
                }),
                None => None,
            },
            multiview: desc.multiview,
        });

        self.pipelines.push(pipeline);
        self.id_to_ind.insert(id, self.pipelines.len() - 1);
    }

    pub fn create_available_in_waiting(
        &mut self,
        render_device: &RenderDevice,
        shaders: &Assets<Shader>,
    ) {
        let waiting_take = std::mem::replace(&mut self.waiting, Vec::new());
        for (id, desc) in waiting_take {
            let Some(vertex_shader) = shaders.get(&desc.vertex.shader) else {
                self.waiting.push((id.clone(), desc.clone()));
                continue;
            };
            let (vf_same, fragment_shader) = match &desc.fragment {
                Some(fragment_state) => {
                    if fragment_state.shader.eq(&desc.vertex.shader) {
                        (true, None)
                    } else {
                        let Some(fragment_shader) = shaders.get(&fragment_state.shader) else {
                            self.waiting.push((id.clone(), desc.clone()));
                            continue;
                        };
                        (false, Some(fragment_shader))
                    }
                }
                None => (false, None),
            };

            let vs_module = vertex_shader.compile(render_device);
            let fs_module = fragment_shader.map(|s| s.compile(render_device));

            self.create(
                render_device,
                id.clone(),
                &desc,
                &vs_module,
                if vf_same {
                    Some(&vs_module)
                } else {
                    fs_module.as_ref()
                },
            );
        }
    }
}

pub fn compile_shaders_into_pipelines(
    render_device: Res<RenderDevice>,
    mut pipeline_cache: ResMut<PipelineCache>,
    shaders: Res<Assets<Shader>>,
) {
    pipeline_cache.create_available_in_waiting(&render_device, &shaders)
}

#[derive(Clone, Debug)]
pub struct RenderPipelineDescriptor {
    /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
    pub label: wgpu::Label<'static>,
    /// The layout of bind groups for this pipeline.
    pub layout: PipelineLayoutDescriptor,
    /// The compiled vertex stage, its entry point, and the input buffers layout.
    pub vertex: VertexState,
    /// The compiled fragment stage, its entry point, and the color targets.
    pub fragment: Option<FragmentState>,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    pub primitive: wgpu::PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    pub multisample: wgpu::MultisampleState,
    /// If the pipeline will be used with a multiview render pass, this indicates how many array
    /// layers the attachments will have.
    pub multiview: Option<NonZeroU32>,
}

#[derive(Clone, Debug)]
pub struct VertexState {
    pub shader: Handle<Shader>,
    pub entry_point: &'static str,
    pub buffers: Vec<wgpu::VertexBufferLayout<'static>>,
}

#[derive(Clone, Debug)]
pub struct FragmentState {
    pub shader: Handle<Shader>,
    pub entry_point: &'static str,
    pub targets: Vec<Option<wgpu::ColorTargetState>>,
}

#[derive(Clone, Debug, Default)]
pub struct PipelineLayoutDescriptor {
    /// Debug label of the pipeline layout. This will show up in graphics debuggers for easy identification.
    pub label: wgpu::Label<'static>,
    /// Bind groups that this pipeline uses. The first entry will provide all the bindings for
    /// "set = 0", second entry will provide all the bindings for "set = 1" etc.
    pub bind_group_layouts: Vec<BindGroupLayout>,
    /// Set of push constant ranges this pipeline uses. Each shader stage that uses push constants
    /// must define the range in push constant memory that corresponds to its single `layout(push_constant)`
    /// uniform block.
    ///
    /// If this array is non-empty, the [`Features::PUSH_CONSTANTS`] must be enabled.
    pub push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

#[derive(Clone, Debug)]
pub struct BindGroupLayout {
    value: Arc<wgpu::BindGroupLayout>,
}

impl From<wgpu::BindGroupLayout> for BindGroupLayout {
    fn from(value: wgpu::BindGroupLayout) -> Self {
        Self {
            value: Arc::new(value),
        }
    }
}

impl Deref for BindGroupLayout {
    type Target = wgpu::BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
