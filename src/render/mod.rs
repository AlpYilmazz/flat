use std::collections::HashMap;

use bevy_app::{CoreStage, Plugin};
use bevy_asset::{AddAsset, Assets, HandleId};
use bevy_ecs::{
    prelude::{Component, EventReader},
    schedule::{StageLabel, SystemStage},
    system::{Commands, Local, Query, Res, ResMut},
    world::World,
};
use wgpu::include_wgsl;
use winit::dpi::PhysicalSize;

use crate::{
    camera::{Camera, CameraUniform, CameraView, PerspectiveProjection},
    texture,
    util::{store, store_many, Refer, ReferMany, Store},
    window::{
        events::{WindowCreated, WindowResized},
        WindowId, WinitWindows,
    },
};

use self::{
    mesh::{GpuMesh, Mesh},
    resource::{
        bind::{BindingSet, UniformBuffer, UpdateGpuUniform},
        shader::{ShaderSource, ShaderSourceLoader, Shaders},
    },
    resource::{
        buffer::{MeshVertex, Vertex},
        pipeline::RenderPipeline,
        shader::Shader,
    },
    system::AddRenderSystem,
};

pub mod mesh;
pub mod mesh_bevy;
pub mod resource;
pub mod system;

#[derive(StageLabel)]
pub enum RenderStage {
    Extract,
    Render,
}

pub struct FlatRenderPlugin;
impl Plugin for FlatRenderPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_stage_after(
            CoreStage::Last,
            RenderStage::Extract,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RenderStage::Extract,
            RenderStage::Render,
            SystemStage::parallel(),
        )
        .init_resource::<Surfaces>()
        .init_resource::<Store<RenderPipeline>>()
        .init_resource::<Store<wgpu::BindGroup>>()
        .init_resource::<Store<(Camera, CameraView, PerspectiveProjection)>>()
        .init_resource::<Store<UniformBuffer<CameraUniform>>>()
        .init_resource::<Shaders>()
        .add_asset_loader(ShaderSourceLoader)
        .add_asset::<ShaderSource>()
        .add_system_to_stage(CoreStage::PreUpdate, create_surface_system)
        .add_system_to_stage(CoreStage::Update, create_render_entity_test)
        .add_system_to_stage(RenderStage::Extract, reconfigure_surface_system)
        .add_render_system::<Mesh<Vertex>>();
        // .add_system_to_stage(RenderStage::Render, render_system);

        create_wgpu_resources(&mut app.world);
        // create_render_entity_test(&mut app.world);
    }
}

// pub struct RenderAsset {
//     pipeline: wgpu::RenderPipeline,
//     bind_groups: Vec<wgpu::BindGroup>,
//     mesh: GpuMesh,
//     instance_data: wgpu::Buffer,
// }

#[derive(Component)]
pub struct InstanceData(wgpu::Buffer, u32);

pub struct DepthTexture(texture::Texture);

pub struct SurfaceKit {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(Default)]
pub struct Surfaces(pub HashMap<WindowId, SurfaceKit>);

pub fn create_wgpu_resources(world: &mut World) {
    let winit_windows = world.get_resource::<WinitWindows>().unwrap();
    let window = winit_windows.map.get(&WindowId::primary()).unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty() | wgpu::Features::TEXTURE_BINDING_ARRAY,
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
        },
        None, // trace_path
    ))
    .unwrap();

    world.insert_resource(instance);
    world.insert_resource(adapter);
    world.insert_resource(device);
    world.insert_resource(queue);
}

fn create_render_entity_test(
    mut count: Local<usize>,
    device: Res<wgpu::Device>,
    surfaces: Res<Surfaces>,
    mut pipelines: ResMut<Store<RenderPipeline>>,
    mut bind_groups: ResMut<Store<wgpu::BindGroup>>,
    mut cameras: ResMut<Store<(Camera, CameraView, PerspectiveProjection)>>,
    mut camera_uniforms: ResMut<Store<UniformBuffer<CameraUniform>>>,
    mut meshes: ResMut<Assets<Mesh<Vertex>>>,
    mut commands: Commands,
) {
    if *count > 0 {
        return;
    }
    *count += 1;

    let config = &surfaces.0.get(&WindowId::primary()).unwrap().config;

    let mut camera = Camera::default();
    let camera_view = CameraView::default();
    let perspective_projection = PerspectiveProjection::default();
    camera.view_matrix = camera_view.build_view_matrix();
    camera.projection_matrix = perspective_projection.build_projection_matrix();
    let mut camera_uniform = CameraUniform::default();
    camera.update_uniform(&mut camera_uniform);
    let camera_uniform = UniformBuffer::new_init(&device, camera_uniform);
    let camera_bind = (&camera_uniform).into_bind_group(&device);

    let pipeline = RenderPipeline::create_usual(
        &device,
        &[&camera_uniform.as_ref().bind_group_layout(&device)],
        &Shader::with_final(
            device.create_shader_module(include_wgsl!("../../res/test.wgsl")),
            vec![Vertex::layout()],
            vec![Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        ),
        wgpu::PrimitiveTopology::TriangleList,
        false,
    );

    let cube_mesh = mesh::primitive::create_unit_cube();

    cameras.insert_primary((camera, camera_view, perspective_projection));
    camera_uniforms.insert_primary(camera_uniform);

    let refer_pipeline = store(&mut pipelines, pipeline);
    let refer_binds = store_many(&mut bind_groups, vec![camera_bind]);
    let handle_cube_mesh = meshes.set(HandleId::random::<Mesh<Vertex>>(), cube_mesh);

    commands
        .spawn()
        .insert(refer_pipeline)
        .insert(refer_binds)
        .insert(handle_cube_mesh);
}

fn reconfigure_camera_aspect(
    queue: &wgpu::Queue,
    camera: &mut Camera,
    camera_uniform: &mut UniformBuffer<CameraUniform>,
    perspective_projection: &mut PerspectiveProjection,
    new_size: &PhysicalSize<u32>,
) {
    perspective_projection.aspect = new_size.width as f32 / new_size.height as f32;
    camera.projection_matrix = perspective_projection.build_projection_matrix();
    let mut camera_gpu_uniform = CameraUniform::default();
    camera.update_uniform(&mut camera_gpu_uniform);
    camera_uniform.update(queue, camera_gpu_uniform);
}

pub fn reconfigure_surface_system(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    mut surfaces: ResMut<Surfaces>,
    mut cameras: ResMut<Store<(Camera, CameraView, PerspectiveProjection)>>,
    mut camera_uniforms: ResMut<Store<UniformBuffer<CameraUniform>>>,
    // window_resize_events: Res<Events<WindowResized>>,
    mut window_resize_events: EventReader<WindowResized>,
) {
    // let mut reader: ManualEventReader<WindowResized> = ManualEventReader::default();
    for WindowResized {
        id: window_id,
        new_size,
    } in window_resize_events.iter()
    {
        if new_size.width > 0 && new_size.height > 0 {
            println!(
                "Reconfiguring window: {:?}, size: {:?}",
                window_id, new_size
            );
            let SurfaceKit { surface, config } = surfaces.0.get_mut(window_id).unwrap();
            config.width = new_size.width;
            config.height = new_size.height;
            surface.configure(&device, config);

            if window_id.is_primary() {
                let (camera, _, perspective_projection) = cameras.get_primary_mut().unwrap();
                reconfigure_camera_aspect(
                    &queue,
                    camera,
                    camera_uniforms.get_primary_mut().unwrap(),
                    perspective_projection,
                    &new_size,
                )
            }
        }
    }
}

pub fn create_surface_system(
    device: Res<wgpu::Device>,
    instance: Res<wgpu::Instance>,
    adapter: Res<wgpu::Adapter>,
    mut surfaces: ResMut<Surfaces>,
    // windows: Res<Windows>,
    winit_windows: Res<WinitWindows>,
    // create_window_events: Res<Events<WindowCreated>>,
    mut create_window_events: EventReader<WindowCreated>,
) {
    // let mut reader: ManualEventReader<WindowCreated> = ManualEventReader::default();
    for WindowCreated { id: window_id } in create_window_events.iter() {
        // TODO: put RawWindowHandle into Windows store
        // let raw_window = &windows.map.get(window_id).unwrap().raw_window_handle;
        // let surface = unsafe { instance.create_surface(&raw_window.get_handle()) };
        let window = winit_windows.map.get(window_id).unwrap();
        let size = window.inner_size();
        println!("Creating window: {:?}, size: {:?}", window_id, size);
        let surface = unsafe { instance.create_surface(window) };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        surfaces
            .0
            .insert(window_id.clone(), SurfaceKit { surface, config });
    }
}

pub fn render_system(
    // surface: Res<wgpu::Surface>,
    surfaces: Res<Surfaces>,
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    depth_texture: Option<Res<DepthTexture>>,
    pipelines: Res<Store<RenderPipeline>>,
    bind_groups: Res<Store<wgpu::BindGroup>>,
    objects: Query<(
        &Refer<RenderPipeline>,
        &ReferMany<wgpu::BindGroup>,
        &GpuMesh,
        Option<&InstanceData>,
    )>,
) {
    let surface = &surfaces.0.get(&WindowId::primary()).unwrap().surface;
    let output = surface.get_current_texture().unwrap();
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: depth_texture.as_ref().as_ref().map(|dt| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: &dt.0.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }
            }),
            // depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            //     view: &(
            //         depth_texture
            //         .as_ref()
            //         .as_ref()
            //         .unwrap()
            //         .0
            //         .view
            //     ),
            //     depth_ops: Some(wgpu::Operations {
            //         load: wgpu::LoadOp::Clear(1.0),
            //         store: true,
            //     }),
            //     stencil_ops: None,
            // }),
        });

        for (pipeline, binds, mesh, instance) in objects.iter() {
            draw_mesh(
                &mut render_pass,
                pipelines.get(**pipeline).unwrap(),
                (*binds)
                    .iter()
                    .map(|i| bind_groups.get(*i).unwrap())
                    .collect::<Vec<_>>(),
                mesh,
                instance,
            );
        }
    } // drop(render_pass) <- mut borrow encoder <- mut borrow self

    queue.submit(std::iter::once(encoder.finish()));

    output.present();
}

fn draw_mesh<'a>(
    render_pass: &mut wgpu::RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_groups: Vec<&'a wgpu::BindGroup>,
    mesh: &'a GpuMesh,
    instance: Option<&'a InstanceData>,
) {
    render_pass.set_pipeline(&pipeline.0);

    // TODO: binds are bound in the same order as they appear in RefMulti
    for (index, bind_group) in bind_groups.into_iter().enumerate() {
        render_pass.set_bind_group(index as u32, bind_group, &[]);
    }

    let mut instance_count = 1;
    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    if let Some(instance_data) = instance {
        render_pass.set_vertex_buffer(1, instance_data.0.slice(..));
        instance_count = instance_data.1;
    }

    match &mesh.assembly {
        mesh::GpuMeshAssembly::Indexed {
            index_buffer,
            index_count,
            index_format,
        } => {
            render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
            render_pass.draw_indexed(0..*index_count as u32, 0, 0..instance_count);
        }
        mesh::GpuMeshAssembly::NonIndexed { vertex_count } => {
            render_pass.draw(0..*vertex_count as u32, 0..instance_count);
        }
    }
}
