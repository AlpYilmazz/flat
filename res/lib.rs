
use crate::legacy::camera::{Camera, PerspectiveProjection, CameraView, CameraUniform};
use cgmath::*;
use resource::{buffer::{Vertex, MeshVertex, Indices, InstanceRaw, InstanceUnit, self}, RenderResources, shader::Shader, mesh, TypedBindGroupLayout, RenderRef};
use crate::legacy::skybox::{self, SkyboxModelUniform};
use crate::legacy::texture;
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{window::Window, event::*};

pub mod legacy;
pub mod util;
pub mod resource;
pub mod camera;
pub mod texture;
pub mod text;


pub struct UserState {
    clear_color: wgpu::Color,
    diffuse_blue_noise: texture::Texture,
    airboat_ref: std::ops::Range<usize>,
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_resources: RenderResources,
    diffuse_texture: texture::Texture,

    camera_uniform: CameraUniform,
    camera_buffer_id: usize,
    camera: Camera,
    camera_view: CameraView,
    camera_projection: PerspectiveProjection,
    camera_controller: CameraController,
    
    skybox_model_uniform: SkyboxModelUniform,
    skybox_model_buffer_id: usize,
    skybox_transform: skybox::SkyboxTransform,

    instances: Vec<buffer::Instance>,
    instance_buffer: wgpu::Buffer,
    render_refs: Vec<(usize, Vec<RenderRef>)>,
    
    pub user_state: UserState,
    framesave_buffer: wgpu::Buffer,
    pub recorded_frames: Vec<Vec<u8>>,
    depth_texture: texture::Texture,
}

impl State {
    const VERTICES: &'static [Vertex] = &[
        Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
        Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
        Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
        Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
        Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
    ];

    const INDICES: &'static [u16] = &[
        0, 1, 4,
        1, 2, 4,
        2, 3, 4,
    ];

    const NUM_INSTANCES_PER_ROW: u32 = 1;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = Vector3::new(
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5
    );

    const BACKGROUND_VERTICES: &'static [Vertex] = &[
        Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] }, // A
        Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // B
        Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // C
        Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] }, // E
    ];

    const BACKGROUND_INDICES: &'static [u16] = &[
        0, 1, 2,
        0, 2, 3,
    ];

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty() | wgpu::Features::TEXTURE_BINDING_ARRAY ,
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None, // trace_path
        ).await.unwrap();
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);
        
        let mut render_resources = RenderResources::empty();
        
        let bytes = include_bytes!("../res/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            bytes,
            "Diffuse Texture",
        ).expect("Texture could not be created");

        let noise_bytes = include_bytes!("../res/noise.png");
        let diffuse_blue_noise = texture::Texture::from_bytes(
            &device,
            &queue,
            noise_bytes,
            "Blue Noise Texture"
        ).expect("Blue Noise texture could not be created");

        let texture_layout = render_resources.just_create_texture_layout(&device);

        let tree_texture_bind_id = 
            render_resources.create_texture_bind_group(
                &device,
                &texture_layout,
                &diffuse_texture
            );

        let noise_texture_bind_id = 
            render_resources.create_texture_bind_group(
                &device,
                &texture_layout,
                &diffuse_blue_noise
            );

        let camera_controller = CameraController::new(0.2);
        let mut camera = Camera::new();
        let static_camera = Camera::new();
        let camera_view = CameraView::default();
        let camera_projection = PerspectiveProjection::default();

        camera.view_matrix = camera_view.build_view_matrix();
        camera.projection_matrix = camera_projection.build_projection_matrix();

        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_proj(&camera);
        
        let mut static_camera_uniform = CameraUniform::default();
        static_camera_uniform.update_view_proj(&static_camera);

        let camera_buffer_id = render_resources.create_uniform_buffer_init(
            &device,
            bytemuck::cast_slice(&[camera_uniform])
        );
        let camera_layout: TypedBindGroupLayout<CameraUniform> = render_resources.just_create_uniform_layout(&device);
        let camera_bind_id = 
            render_resources.create_uniform_bind_group(
                &device,
                &camera_layout,
                camera_buffer_id
            );
        
        let static_camera_buffer_id = render_resources.create_uniform_buffer_init(
            &device,
            bytemuck::cast_slice(&[static_camera_uniform])
        );
        let static_camera_bind_id = 
            render_resources.create_uniform_bind_group(
                &device,
                &camera_layout,
                static_camera_buffer_id
            );

        let basic_wgsl = device.create_shader_module(include_wgsl!("../res/basic.wgsl"));
        let basic_wgsl_pipeline_id = render_resources.create_render_pipeline(
            &device,
            &[
                &texture_layout,
                &camera_layout,
            ],
            &Shader::with_final(
                basic_wgsl,
                vec![Vertex::layout(), InstanceRaw::layout()],
                vec![Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            ),
            wgpu::PrimitiveTopology::TriangleList
        );

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            &config,
            "Depth Texture",
        );

        let cube_mesh = mesh::primitive::create_unit_cube();
        let cube_mesh_id = render_resources.create_gpu_mesh(
            &device,
            &cube_mesh
        );

        let mut plane_mesh = mesh::primitive::create_aa_plane(
            mesh::primitive::PlaneAlign::XZ,
            20.0, 20.0, 20, 20, Vector3::zero(),
        );
        mesh::util::randomize_y(&mut plane_mesh);
        let plane_mesh_id = render_resources.create_gpu_mesh(
            &device,
            &plane_mesh
        );

        let pentagon_mesh: mesh::Mesh<Vertex> = 
            mesh::Mesh::with_all(
                wgpu::PrimitiveTopology::TriangleList,
                Self::VERTICES.to_owned(),
                Some(Indices::U16(Self::INDICES.to_owned())),
            );
        let pentagon_mesh_id = render_resources.create_gpu_mesh(
            &device,
            &pentagon_mesh
        );
        
        let background_mesh: mesh::Mesh<Vertex> = 
        mesh::Mesh::with_all(
            wgpu::PrimitiveTopology::TriangleList,
            Self::BACKGROUND_VERTICES.to_owned(),
            Some(Indices::U16(Self::BACKGROUND_INDICES.to_owned())),
        );
        let background_mesh_id = render_resources.create_gpu_mesh(
            &device,
            &background_mesh
        );
        
        let model: mesh::Model<Vertex> = mesh::Mesh::load_obj("res/airboat.obj");
        let model_mesh_count = model.meshes.len();
        let mut maxi = 0;
        for mesh in model.meshes {
            maxi = render_resources.create_gpu_mesh(&device, &mesh);
        }
        let airboat_ref = maxi - model_mesh_count + 1 .. maxi+1;

        let instances = (0..Self::NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..Self::NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - Self::INSTANCE_DISPLACEMENT;

                let position = cgmath::Vector3 {
                    x: 0.0, y: 0.0, z: 0.0
                };

                let scale = cgmath::Vector3 {
                    x: 1.5, y: 3.0, z: 1.0
                };

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly
                    Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                buffer::Instance {
                    position, scale, rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances
            .iter()
            .map(|ins| {
                ins.to_raw()
            })
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let skybox_mesh = skybox::create_skybox();
        let skybox_mesh_id = render_resources.create_gpu_mesh(&device, &skybox_mesh);

        let skybox_texture_array = texture::TextureArray::load(
            &device, &queue, "res/skybox", skybox::SIDES, "jpg"
        ).unwrap();
        let texture_array_6_layout: TypedBindGroupLayout<texture::TextureArray<6>> = 
            render_resources.just_create_bind_group_layout(&device);
        let texture_array_6_bind_id = render_resources.create_texture_array_bind_group(
            &device, &texture_array_6_layout, &skybox_texture_array
        );

        let skybox_transform = skybox::SkyboxTransform {
            scale: Vector3::new(500.0, 500.0, 500.0),
            ..Default::default()
        };
        let mut skybox_model_uniform = skybox::SkyboxModelUniform::default();
        skybox_model_uniform.update(&skybox_transform);
        let skybox_model_buffer_id = render_resources.create_uniform_buffer_init(
            &device,
            bytemuck::cast_slice(&[skybox_model_uniform])
        );
        let skybox_model_layout: TypedBindGroupLayout<SkyboxModelUniform> = 
            render_resources.just_create_uniform_layout(&device);
        let skybox_model_bind_id = 
            render_resources.create_uniform_bind_group(
                &device,
                &skybox_model_layout,
                skybox_model_buffer_id
            );

        let skybox_render_pipeline = skybox::create_skybox_render_pipeline(
            &render_resources, &device, &config
        );
        let skybox_pipeline_id = render_resources.push_render_pipeline(skybox_render_pipeline);
  
        let font_arial = String::from("arial.ttf");
        let mut text_map = text::TextMap::new();
        text_map.generate(font_arial.clone(), 0).unwrap();
        
        let arial_fontc = text_map.fonts.get(&font_arial).unwrap();
        let arial_font_tex_bytes = &arial_fontc.atlas.bytes;
        let arial_font_atlas_texture = texture::Texture::from_raw_image(
            &device, &queue,
            &texture::RawImage::new( 
                arial_font_tex_bytes,
                (arial_fontc.atlas.h as u32, arial_fontc.atlas.w as u32),
                texture::PixelFormat::G8,
            ),
            None,
        ).unwrap();
        let arial_font_atlas_bind_id = render_resources.create_texture_bind_group(
            &device,
            &texture_layout,
            &arial_font_atlas_texture
        );

        let text_hello_mesh = text::mesh::create_screen_text_mesh(
            &arial_fontc.atlas,
            "Hello, World!",
            (0.0, 0.0),
        );
        let text_hello_mesh_id = render_resources.create_gpu_mesh(&device, &text_hello_mesh);

        let mut camera_ortho_uniform = CameraUniform::default();
        let camera_ortho_buffer_id = render_resources.create_uniform_buffer_init(
            &device,
            bytemuck::cast_slice(&[camera_ortho_uniform])
        );
        let camera_ortho_bind_id = 
            render_resources.create_uniform_bind_group(
                &device,
                &camera_layout,
                camera_ortho_buffer_id
            );

        let text_wgsl = device.create_shader_module(include_wgsl!("../res/text.wgsl"));
        let text_render_pipeline = render_resources.create_render_pipeline(
            &device,
            &[
                &texture_layout,
                &camera_layout,
            ],
            &Shader::with_final(
                text_wgsl,
                vec![Vertex::layout()],
                vec![Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            ),
            wgpu::PrimitiveTopology::TriangleList,
        );


        let text_hello = RenderRef {
            pipeline: text_render_pipeline,
            mesh: text_hello_mesh_id,
            bind_groups: vec![
                arial_font_atlas_bind_id,
                camera_ortho_bind_id,
                // text_model_bind_id,
            ],
        };

        let skybox = RenderRef {
            pipeline: skybox_pipeline_id,
            mesh: skybox_mesh_id,
            bind_groups: vec![
                texture_array_6_bind_id,
                camera_bind_id,
                skybox_model_bind_id,
            ],
        };

        let cube = RenderRef {
            pipeline: basic_wgsl_pipeline_id,
            mesh: cube_mesh_id,
            bind_groups: vec![
                tree_texture_bind_id,
                camera_bind_id,
            ],
        };

        let plane = RenderRef {
            pipeline: basic_wgsl_pipeline_id,
            mesh: plane_mesh_id,
            bind_groups: vec![
                tree_texture_bind_id,
                camera_bind_id,
            ],
        };

        let pentagon = RenderRef {
            pipeline: basic_wgsl_pipeline_id,
            mesh: pentagon_mesh_id,
            bind_groups: vec![
                tree_texture_bind_id,
                camera_bind_id,
            ],
        };

        let background = RenderRef {
            pipeline: basic_wgsl_pipeline_id,
            mesh: background_mesh_id,
            bind_groups: vec![
                noise_texture_bind_id,
                static_camera_bind_id,
            ],
        };

        let u32_size = std::mem::size_of::<u32>() as u32;
        let bs_offset = 76800;
        let framesave_buffer_size = (u32_size * size.width * size.height + bs_offset) as wgpu::BufferAddress;
        let framesave_buffer_desc = wgpu::BufferDescriptor {
            size: framesave_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let framesave_buffer = device.create_buffer(&framesave_buffer_desc);

        let user_state = UserState {
            clear_color: wgpu::Color::BLACK,
            diffuse_blue_noise,
            airboat_ref,
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_resources,
            depth_texture,
            diffuse_texture,
            
            camera_uniform,
            camera_buffer_id,
            camera,
            camera_view,
            camera_projection,
            camera_controller,

            skybox_model_uniform,
            skybox_model_buffer_id,
            skybox_transform,

            instances,
            instance_buffer,

            render_refs: vec![
                (
                    basic_wgsl_pipeline_id,
                    vec![
                        // background,
                        // pentagon,
                        // plane,
                        // cube,
                    ]
                ),
                (
                    skybox_pipeline_id,
                    vec![
                        skybox,
                    ]
                )
            ],

            framesave_buffer,
            recorded_frames: Vec::with_capacity(size.height as usize),

            user_state,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size; // Copy
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.config,
                "Depth Texture",
            );
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event);
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.user_state.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };

                true
            },
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::R),
                    ..
                },
                ..
            } => {
                save_gif(
                    "save/record.gif",
                    &mut self.recorded_frames,
                    10,
                    self.size.width as u16,
                    self.size.height as u16
                ).unwrap();

                true
            },
            _ => false,
        }
    }

    pub fn update(&mut self) {
        // self.camera_view.eye += (0.0, 0.1, 0.1).into();
        self.camera_controller.update_camera_view(&mut self.camera_view);

        self.camera.view_matrix = self.camera_view.build_view_matrix();
        self.camera_uniform.update_view_proj(&self.camera);
        let camera_buffer = self.render_resources.buffers.get(self.camera_buffer_id).unwrap();
        self.queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        // TODO: abstract/unite uniform_repr-uniform-buffer update
        self.skybox_transform.translation = self.camera_view.eye.to_vec();
        self.skybox_model_uniform.update(&self.skybox_transform);
        let skybox_buffer = self.render_resources.buffers.get(self.skybox_model_buffer_id).unwrap();
        self.queue.write_buffer(skybox_buffer, 0, bytemuck::cast_slice(&[self.skybox_model_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.user_state.clear_color), // Copy
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                }
            );

            for (pipeline_id, render_refs) in &self.render_refs {
                self.set_pipeline(&mut render_pass, *pipeline_id);
                for render_ref in render_refs {   
                    self.render_pass_draw(&mut render_pass, render_ref);
                }
            }

        } // drop(render_pass) <- mut borrow encoder <- mut borrow self
        
        
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    fn set_pipeline<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        render_pipeline_id: usize,
    ) {
        let render_pipeline = 
            &self.render_resources.render_pipelines[render_pipeline_id];
            
        render_pass.set_pipeline(render_pipeline);
    }

    fn render_pass_draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        render_ref: &RenderRef,
    ) {
        let mesh_0 = 
            &self.render_resources.meshes[render_ref.mesh];

        for (index, bind_group_id) in (&render_ref.bind_groups).into_iter().enumerate() {
            let bind_group = &self.render_resources.bind_groups[*bind_group_id];
            render_pass.set_bind_group(index as u32, bind_group, &[]);
        }
        render_pass.set_vertex_buffer(0, mesh_0.vertex_buffer.slice(..));
        // TODO:
        // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        match &mesh_0.assembly {
            mesh::GpuMeshAssembly::Indexed {
                index_buffer,
                index_count,
                index_format
            } => {
                render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
                render_pass.draw_indexed(0..*index_count as u32, 0, 0..self.instances.len() as u32);
            },
            mesh::GpuMeshAssembly::NonIndexed {
                vertex_count
            } => {
                render_pass.draw(0..*vertex_count as u32, 0..self.instances.len() as u32);
            },
        }
    }
}


struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LControl => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn simple_update_camera_view(&self, camera_view: &mut CameraView) {
        if self.is_forward_pressed {
            let step = -Vector3::unit_z() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
        if self.is_left_pressed {
            let step = -Vector3::unit_x() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
        if self.is_backward_pressed {
            let step = Vector3::unit_z() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
        if self.is_right_pressed {
            let step = Vector3::unit_x() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
        if self.is_up_pressed {
            let step = Vector3::unit_y() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
        if self.is_down_pressed {
            let step = -Vector3::unit_y() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
    }

    fn update_camera_view(&self, camera_view: &mut CameraView) {
        // use cgmath::InnerSpace;
        let forward = camera_view.target - camera_view.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            let step = forward_norm * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
        if self.is_backward_pressed {
            let step = -forward_norm * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }

        if self.is_up_pressed {
            let step = Vector3::unit_y() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }
        if self.is_down_pressed {
            let step = -Vector3::unit_y() * self.speed;
            camera_view.eye += step;
            camera_view.target += step;
        }

        let right = forward_norm.cross(camera_view.up);

        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = camera_view.target - camera_view.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so 
            // that it doesn't change. The eye therefore still 
            // lies on the circle made by the target and eye.
            camera_view.eye = camera_view.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera_view.eye = camera_view.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}


// let pixel_size = std::mem::size_of::<[u8;4]>() as u32;
//         let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
//         let unpadded_bytes_per_row = pixel_size * self.size.width;
//         let padding = (align - unpadded_bytes_per_row % align) % align;
//         let padded_bytes_per_row = unpadded_bytes_per_row + padding;

//         // println!("{}\n{}\n{}\n", padded_bytes_per_row, self.size.height, 
//         //     padded_bytes_per_row * self.size.height);

//         let frame = output.texture.as_image_copy();
//         encoder.copy_texture_to_buffer(
//             frame,
//             wgpu::ImageCopyBuffer {
//                 buffer: &self.framesave_buffer,
//                 layout: wgpu::ImageDataLayout {
//                     offset: 0,
//                     bytes_per_row: NonZeroU32::new(padded_bytes_per_row),
//                     rows_per_image: NonZeroU32::new(self.size.height),
//                 },
//             },
//             wgpu::Extent3d {
//                 width: self.size.width,
//                 height: self.size.height,
//                 depth_or_array_layers: 1,
//             },
//         );

//         let buffer_slice = self.framesave_buffer.slice(..);
//         let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
//         buffer_slice.map_async(
//             wgpu::MapMode::Read, 
//             move |result| {
//                 tx.send(result).unwrap();
//             }
//         );
//         // wait for the GPU to finish
//         self.device.poll(wgpu::Maintain::Wait);

//         let result = pollster::block_on(rx.receive());

//         match result {
//             Some(Ok(())) => {
//                 let padded_data = buffer_slice.get_mapped_range();
//                 let data = padded_data
//                     .chunks(padded_bytes_per_row as _)
//                     .map(|chunk| &chunk[..unpadded_bytes_per_row as _])
//                     .flatten()
//                     .map(|x| *x)
//                     .collect::<Vec<_>>();
//                 drop(padded_data);
//                 self.framesave_buffer.unmap();
//                 self.recorded_frames.push(data);
//             }
//             _ => eprintln!("Something went wrong"),
//         }

fn save_gif(path: &str, frames: &mut Vec<Vec<u8>>, speed: i32, w: u16, h: u16) -> anyhow::Result<()> {
    use gif::{Encoder, Frame, Repeat};

    let mut image = std::fs::File::create(path)?;
    let mut encoder = Encoder::new(&mut image, w, h, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for mut frame in frames {
        encoder.write_frame(&Frame::from_rgba_speed(w, h, &mut frame, speed))?;
    }

    Ok(())
}