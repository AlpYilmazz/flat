use bevy_app::App;
use bevy_ecs::{
    prelude::Entity,
    system::{
        lifetimeless::{Read, SQuery, SRes, SResMut},
        SystemParam, SystemParamItem, SystemState, ReadOnlySystemParamFetch,
    }, world::World,
};

use super::{
    mesh::GpuMesh,
    system::{ExtractedAssets, RenderHandle, Draw},
};

pub trait DrawFunction: Send + Sync + 'static {
    fn draw<'w>(
        &mut self,
        world: &'w World,
        render_pass: &mut wgpu::RenderPass<'w>,
        entity: Entity,
    );
}

pub trait RenderCommand: Send + Sync + 'static {
    type Param: SystemParam;

    fn execute<'w>(
        render_pass: &mut wgpu::RenderPass<'w>,
        entity: Entity,
        param: SystemParamItem<'w, '_, Self::Param>,
    );
}

pub struct DrawGpuMesh;
impl RenderCommand for DrawGpuMesh {
    type Param = (
        SResMut<ExtractedAssets<GpuMesh>>,
        SQuery<Read<RenderHandle<GpuMesh>>>,
    );

    fn execute<'w>(
        render_pass: &mut wgpu::RenderPass<'w>,
        entity: Entity,
        (gpu_meshes, mesh_handles): SystemParamItem<'w, '_, Self::Param>,
    ) {
        let mesh_handle = mesh_handles.get(entity).unwrap();
        let gpu_mesh = gpu_meshes.into_inner().get_mut(&mesh_handle.id()).unwrap();

        gpu_mesh.draw(render_pass, None);
    }
}

#[allow(non_snake_case)]
impl<B0: RenderCommand, B1: RenderCommand> RenderCommand for (B0, B1) {
    type Param = (B0::Param, B1::Param);

    fn execute<'w>(
        render_pass: &mut wgpu::RenderPass<'w>,
        entity: Entity,
        (B0_param, B1_param): SystemParamItem<'w, '_, Self::Param>,
    ) {
        B0::execute(render_pass, entity, B0_param);
        B1::execute(render_pass, entity, B1_param);
    }
}

pub struct RenderCommandState<C: RenderCommand> {
    state: SystemState<C::Param>,
}
impl<C: RenderCommand> RenderCommandState<C> {
    pub fn new(world: &mut World) -> Self {
        Self {
            state: SystemState::new(world),
        }
    }
}

impl<C: RenderCommand> DrawFunction for RenderCommandState<C>
where
    <C::Param as SystemParam>::Fetch: ReadOnlySystemParamFetch,
{
    fn draw<'w>(
        &mut self,
        world: &'w World,
        render_pass: &mut wgpu::RenderPass<'w>,
        entity: Entity,
    ) {
        let param = self.state.get(world);
        C::execute(render_pass, entity, param);
    }
}

pub struct DrawFunctionId(pub usize);

pub struct DrawFunctions {
    store: Vec<Box<dyn DrawFunction>>,
}

pub trait AddRenderCommand {
    
}
impl AddRenderCommand for App {

}