use bevy::prelude::{FromWorld, Res, ResMut, Resource};
use encase::ShaderType;

use crate::render::{
    camera::component::CameraUniforms,
    resource::{
        component_uniform::{ComponentUniforms, ModelUniform},
        pipeline::BindGroupLayout,
        renderer::RenderDevice,
    },
};

#[derive(Resource)]
pub struct PipelineCommons {
    pub model_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
}

impl FromWorld for PipelineCommons {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

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
                        min_binding_size: Some(CameraUniforms::min_size()),
                    },
                    count: None,
                }],
                label: Some("sprite_view_layout"),
            });

        PipelineCommons {
            model_layout,
            view_layout,
        }
    }
}

#[derive(Default, Resource)]
pub struct CommonBindGroups {
    pub model_bind_group: Option<wgpu::BindGroup>,
    pub view_bind_group: Option<wgpu::BindGroup>,
}

pub fn create_common_bind_groups(
    pipeline_commons: Res<PipelineCommons>,
    mut common_bind_groups: ResMut<CommonBindGroups>,
    render_device: Res<RenderDevice>,
    model_uniforms: Res<ComponentUniforms<ModelUniform>>,
    view_uniforms: Res<ComponentUniforms<CameraUniforms>>,
) {
    let Some(model_binding) = model_uniforms.binding() else {
        return;
    };
    let model_bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline_commons.model_layout,
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
        layout: &pipeline_commons.view_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: view_binding,
        }],
    });

    common_bind_groups.model_bind_group = Some(model_bind_group);
    common_bind_groups.view_bind_group = Some(view_bind_group);
}
