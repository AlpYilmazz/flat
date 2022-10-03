use bytemuck::{Pod, Zeroable};
use cgmath::{Vector3, Matrix4, SquareMatrix, Zero};

use crate::{resource::{buffer::{MeshVertex, Uniform, Indices}, shader, RenderResources, TypedBindGroupLayout, mesh::Mesh}};

use crate::legacy::{texture, camera::CameraUniform};


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct VertexSkybox {
    pub position: [f32; 3],
    pub tex_index: i32,
    pub tex_coords: [f32; 2],
}

impl MeshVertex for VertexSkybox {
    const ATTR_NAMES: &'static [&'static str] = 
        &[
            "Position",
            "Texture Index",
            "Texture Coordinates",
        ];
    
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = 
        &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Sint32,
            2 => Float32x2,
        ];
}

pub struct SkyboxTransform {
    pub translation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl Default for SkyboxTransform {
    fn default() -> Self {
        Self {
            translation: Vector3::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyboxModelUniform {
    pub model: [[f32; 4]; 4],
}

impl SkyboxModelUniform {
    pub fn update(&mut self, transform: &SkyboxTransform) {
        self.model = (
            Matrix4::from_translation(transform.translation)
            * Matrix4::from_nonuniform_scale(transform.scale.x, transform.scale.y, transform.scale.z) 
        ).into()
    }
}

impl Default for SkyboxModelUniform {
    fn default() -> Self {
        Self {
            model: Matrix4::identity().into(),
        }
    }
}

impl Uniform for SkyboxModelUniform {
    const ENTRIES: &'static [wgpu::BindGroupLayoutEntry] = 
        &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ];
}

pub const SIDES: [&'static str; 6] = [
    // "negy", "posz", "posx",
    // "negz", "negx", "posy",
    "negy", "posz", "posx",
    "negz", "negx", "posy",
];

pub fn create_skybox() -> Mesh<VertexSkybox> {
    // z grows towards, out of the screen
    // +z .. |screen| .. -z
    // TODO: correct the texture coordinates
    const VERTICES_Z_TOWARDS: &[VertexSkybox] = &[
        // Down, -y, negy
        VertexSkybox { position: [-0.5, -0.5, 0.5], tex_index: 0, tex_coords: [0.0, 1.0] }, // 0
        VertexSkybox { position: [-0.5, -0.5, -0.5], tex_index: 0, tex_coords: [0.0, 0.0] }, // 3
        VertexSkybox { position: [0.5, -0.5, -0.5], tex_index: 0, tex_coords: [1.0, 0.0] }, // 2
        VertexSkybox { position: [0.5, -0.5, 0.5], tex_index: 0, tex_coords: [1.0, 1.0] }, // 1

        // Front, +z, posz
        VertexSkybox { position: [-0.5, 0.5, 0.5], tex_index: 1, tex_coords: [1.0, 0.0] }, // 4
        VertexSkybox { position: [-0.5, -0.5, 0.5], tex_index: 1, tex_coords: [1.0, 1.0] }, // 0
        VertexSkybox { position: [0.5, -0.5, 0.5], tex_index: 1, tex_coords: [0.0, 1.0] }, // 1
        VertexSkybox { position: [0.5, 0.5, 0.5], tex_index: 1, tex_coords: [0.0, 0.0] }, // 5

        // Right, +x, posx
        VertexSkybox { position: [0.5, 0.5, 0.5], tex_index: 2, tex_coords: [1.0, 0.0] }, // 5
        VertexSkybox { position: [0.5, -0.5, 0.5], tex_index: 2, tex_coords: [1.0, 1.0] }, // 1
        VertexSkybox { position: [0.5, -0.5, -0.5], tex_index: 2, tex_coords: [0.0, 1.0] }, // 2
        VertexSkybox { position: [0.5, 0.5, -0.5], tex_index: 2, tex_coords: [0.0, 0.0] }, // 6

        // Back, -z, negz
        VertexSkybox { position: [0.5, 0.5, -0.5], tex_index: 3, tex_coords: [1.0, 0.0] }, // 6
        VertexSkybox { position: [0.5, -0.5, -0.5], tex_index: 3, tex_coords: [1.0, 1.0] }, // 2
        VertexSkybox { position: [-0.5, -0.5, -0.5], tex_index: 3, tex_coords: [0.0, 1.0] }, // 3
        VertexSkybox { position: [-0.5, 0.5, -0.5], tex_index: 3, tex_coords: [0.0, 0.0] }, // 7

        // Left, -x, negx
        VertexSkybox { position: [-0.5, 0.5, -0.5], tex_index: 4, tex_coords: [1.0, 0.0] }, // 7
        VertexSkybox { position: [-0.5, -0.5, -0.5], tex_index: 4, tex_coords: [1.0, 1.0] }, // 3
        VertexSkybox { position: [-0.5, -0.5, 0.5], tex_index: 4, tex_coords: [0.0, 1.0] }, // 0
        VertexSkybox { position: [-0.5, 0.5, 0.5], tex_index: 4, tex_coords: [0.0, 0.0] }, // 4
        
        // Up, +y, posy
        VertexSkybox { position: [-0.5, 0.5, -0.5], tex_index: 5, tex_coords: [0.0, 1.0] }, // 7
        VertexSkybox { position: [-0.5, 0.5, 0.5], tex_index: 5, tex_coords: [0.0, 0.0] }, // 4
        VertexSkybox { position: [0.5, 0.5, 0.5], tex_index: 5, tex_coords: [1.0, 0.0] }, // 5
        VertexSkybox { position: [0.5, 0.5, -0.5], tex_index: 5, tex_coords: [1.0, 1.0] }, // 6
    ];

    let mut indices = vec![0; 36];
    for i in 0..6 {
        let range = 6*i .. 6*(i+1);
        indices[range.clone()].copy_from_slice(&[0, 2, 1, 2, 0, 3]);
        for u in &mut indices[range] {
            *u += 4*i as u16;
        }
    }

    Mesh::with_all(
        wgpu::PrimitiveTopology::TriangleList,
        VERTICES_Z_TOWARDS.to_owned(),
        Some(Indices::U16(indices)),
    )
}


pub fn create_skybox_render_pipeline(
    render_resources: &RenderResources,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    // shader_path: &str,
) -> wgpu::RenderPipeline {
    todo!()
    // let shader_module = device.create_shader_module(
    //     // wgpu::ShaderModuleDescriptor {
    //     //     label: None,
    //     //     source: wgpu::ShaderSource::Wgsl(include_str!(shader_path)),
    //     // }
    //     wgpu::include_wgsl!("../../res/skybox.wgsl")
    // );
    // let shader = shader::Shader::with_final(
    //     shader_module,
    //     vec![VertexSkybox::layout()],
    //     vec![Some(wgpu::ColorTargetState {
    //         format: config.format,
    //         blend: Some(wgpu::BlendState::REPLACE),
    //         write_mask: wgpu::ColorWrites::ALL,
    //     })]
    // );

    // let texture_array_layout: TypedBindGroupLayout<texture::TextureArray<6>> = 
    //     render_resources.just_create_bind_group_layout(device);
    // let camera_layout: TypedBindGroupLayout<CameraUniform> = 
    //     render_resources.just_create_uniform_layout(device);
    // let model_matrix_layout: TypedBindGroupLayout<SkyboxModelUniform> = 
    //     render_resources.just_create_uniform_layout(device);

    // let render_pipeline_layout = device.create_pipeline_layout(
    //     &wgpu::PipelineLayoutDescriptor {
    //         label: Some("Render Pipeline Layout"),
    //         bind_group_layouts: &[
    //             &texture_array_layout,
    //             &camera_layout,
    //             &model_matrix_layout,
    //         ],
    //         push_constant_ranges: &[],
    //     }
    // );
    // let render_pipeline = device.create_render_pipeline(
    //     &wgpu::RenderPipelineDescriptor {
    //         label: Some("Render Pipeline"),
    //         layout: Some(&render_pipeline_layout),
    //         vertex: wgpu::VertexState {
    //             module: &shader.module,
    //             entry_point: shader::Shader::VERTEX_ENTRY_POINT,
    //             buffers: &shader.vertex_buffers,
    //         },
    //         fragment: Some(wgpu::FragmentState {
    //             module: &shader.module,
    //             entry_point: shader::Shader::FRAGMENT_ENTRY_POINT,
    //             targets: &shader.fragment_targets,
    //         }),
    //         primitive: wgpu::PrimitiveState {
    //             topology: wgpu::PrimitiveTopology::TriangleList,
    //             strip_index_format: None,
    //             front_face: wgpu::FrontFace::Ccw,
    //             cull_mode: Some(wgpu::Face::Back),
    //             // Setting this to anything other than Fill requires
    //             // Features::NON_FILL_POLYGON_MODE
    //             polygon_mode: wgpu::PolygonMode::Fill,
    //             // Requires Features::DEPTH_CLIP_CONTROL
    //             unclipped_depth: false,
    //             // Requires Features::CONSERVATIVE_RASTERIZATION
    //             conservative: false,
    //         },
    //         depth_stencil: Some(wgpu::DepthStencilState {
    //             format: texture::Texture::DEPTH_FORMAT,
    //             depth_write_enabled: true,
    //             depth_compare: wgpu::CompareFunction::Less, // 1.
    //             stencil: wgpu::StencilState::default(), // 2.
    //             bias: wgpu::DepthBiasState::default(),
    //         }),
    //         multisample: wgpu::MultisampleState {
    //             count: 1,
    //             mask: !0,
    //             alpha_to_coverage_enabled: false,
    //         },
    //         multiview: None,
    //     }
    // );

    // render_pipeline
}