use core::panic;

use bevy::{
    ecs::system::lifetimeless::Read,
    prelude::{
        App, Component, Entity, FromWorld, GlobalTransform, Handle, Mut, QueryState, Resource,
        Transform, With, World, warn, info,
    },
    utils::{HashSet, HashMap},
    window::WindowId,
};
use winit::window::Window;

use super::{
    camera::component::*, color::Color, mesh::Mesh, resource::buffer::MeshVertex, texture::Image,
    view::window::PreparedWindows, RenderAssets, RenderDevice, RenderInstance, RenderQueue,
};

pub struct MeshBundle<V: MeshVertex> {
    pub mesh: Handle<Mesh<V>>,  // Mesh<V>: RenderAsset => GpuMesh
    pub texture: Handle<Image>, // Image: RenderAsset => GpuTexture: CreateBindGroup => BindGroup
    pub transform: Transform,
    pub global_transform: GlobalTransform, // GlobalTransform: DynamicUniform => DynamicUniform::push
    pub color: Color,                      // Color: DynamicUniform => DynamicUniform::push

                                           // pub pipeline_id: CachedRenderPipelineId,
}

pub fn render_system(world: &mut World) {
    world.resource_scope(|world: &mut World, mut render_node: Mut<RenderNode>| {
        render_node.update(&world);
    });

    let render_node = world.get_resource::<RenderNode>().unwrap();
    render_node.run(&world);

    world.resource_scope(|_world: &mut World, mut windows: Mut<PreparedWindows>| {
        for window in windows.values_mut() {
            window.surface_texture.take().unwrap().texture.present();
        }
    });
}

#[derive(Resource)]
pub struct RenderNode {
    cameras: QueryState<(Entity, Read<Camera>, Read<VisibleEntities>)>,
    entities: QueryState<(Entity,), (With<Visibility>,)>,
}

impl FromWorld for RenderNode {
    fn from_world(world: &mut World) -> Self {
        Self::new(world)
    }
}

impl RenderNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            cameras: world.query(),
            entities: world.query_filtered(),
        }
    }

    pub fn update(&mut self, world: &World) {
        self.cameras.update_archetypes(world);
        self.entities.update_archetypes(world);
    }

    pub fn run(&self, world: &World) {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let render_queue = world.get_resource::<RenderQueue>().unwrap();

        let gpu_textures = world.get_resource::<RenderAssets<Image>>().unwrap();
        let windows = world.get_resource::<PreparedWindows>().unwrap();

        let mut command_encoder = render_device.create_command_encoder(&Default::default());

        let render_functions = world.get_resource::<RenderFunctions>().unwrap();
        let cameras = self.cameras.iter_manual(world);

        let mut camera_windows: HashSet<WindowId> = HashSet::new();

        for (camera_entity, camera, visible_entities) in cameras {
            if let Some(id) = camera.render_target.get_window() {
                camera_windows.insert(id);
            }

            let render_target_view = camera.render_target.get_view(&gpu_textures, &windows);

            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            for entity in visible_entities.iter() {
                if let Some(render_function_id) = world.get::<RenderFunctionId>(*entity) {
                    let render = render_functions.get(render_function_id).unwrap();
                    let render_result = (render)(camera_entity, *entity, world, &mut render_pass);
                    match render_result {
                        RenderResult::Success => info!("RenderResult::Success"),
                        RenderResult::Failure => warn!("RenderResult::Failure"),
                    }
                }
            }
        }

        for window in windows
            .values()
            .filter(|window| !camera_windows.contains(&window.id))
        {
            let surface_data = &window.surface_texture.as_ref().unwrap();
            let _render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_data.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        render_queue.submit([command_encoder.finish()]);
    }
}

pub trait AddRenderFunction {
    fn add_render_function(&mut self, id: usize, render: RenderFunction) -> &mut Self;
}
impl AddRenderFunction for App {
    fn add_render_function(&mut self, id: usize, render: RenderFunction) -> &mut Self {
        self.world
            .get_resource_mut::<RenderFunctions>()
            .unwrap()
            .add(RenderFunctionId(id), render);
        self
    }
}

pub enum RenderResult {
    Success,
    Failure,
}

pub type RenderFunction = for<'w> fn(
    /*camera*/ Entity,
    /*object*/ Entity,
    &'w World,
    &mut wgpu::RenderPass<'w>,
) -> RenderResult;

// TODO: entity has to register a RenderFunctionId
//       how does it find the id
#[derive(Component, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderFunctionId(usize);

impl From<usize> for RenderFunctionId {
    fn from(value: usize) -> Self {
        RenderFunctionId(value)
    }
}

#[derive(Resource)]
pub struct RenderFunctions {
    id_to_ind: HashMap<RenderFunctionId, usize>,
    functions: Vec<RenderFunction>,
}

impl Default for RenderFunctions {
    fn default() -> Self {
        Self {
            id_to_ind: HashMap::new(),
            functions: Vec::new(),
        }
    }
}

impl RenderFunctions {
    pub fn add(&mut self, id: RenderFunctionId, render: RenderFunction) {
        if self.id_to_ind.contains_key(&id) {
            panic!("Attempted adding multiple render functions with the same id");
        }
        self.functions.push(render);
        self.id_to_ind.insert(id, self.functions.len() - 1);
    }

    pub fn get(&self, index: &RenderFunctionId) -> Option<&RenderFunction> {
        self.functions.get(*self.id_to_ind.get(index)?)
    }
}

fn unimpl_create<T>() -> T {
    unimplemented!()
}
fn unimpl_from_world<'w, T>(_world: &'w World) -> &'w T {
    unimplemented!()
}

pub fn render_note(world: &World) {
    let window = unimpl_create::<Window>();

    let instance = world.get_resource::<RenderInstance>().unwrap();
    let device = world.get_resource::<RenderDevice>().unwrap();
    let queue = world.get_resource::<RenderQueue>().unwrap();

    let surface = unsafe { instance.create_surface(&window) };
    let surface_texture = surface.get_current_texture().unwrap();
    let surface_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut command_encoder = device.create_command_encoder(&Default::default());
    let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &surface_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    // DO WORK WITH THE RENDER PASS
    let render_functions = unimpl_from_world::<Vec<RenderFunction>>(&world);
    for render_function in render_functions.iter() {
        (render_function)(
            Entity::from_raw(1),
            Entity::from_raw(1),
            &world,
            &mut render_pass,
        );
    }
    // ============================
    drop(render_pass);

    queue.submit([command_encoder.finish()]);
    surface_texture.present();
}
