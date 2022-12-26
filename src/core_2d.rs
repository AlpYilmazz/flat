use std::ops::Range;

use bevy::{
    ecs::system::lifetimeless::Read,
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{
            CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions, EntityPhaseItem,
            PhaseItem, RenderPhase, TrackedRenderPass,
        },
        render_resource::CachedRenderPipelineId,
        renderer::RenderContext,
        view::{ViewTarget, VisibleEntities},
        Extract, RenderApp, RenderStage, camera::{CameraRenderGraph, CameraProjection}, primitives::Frustum,
    },
    utils::FloatOrd,
};

pub mod graph {
    pub const NAME: &'static str = "core_2d";
    pub mod main {
        pub const NODE: &'static str = "main_node";
    }
    pub mod input {
        pub const IN_VIEW: &'static str = "in_view";
    }
}

pub struct FlatCore2dPlugin;
impl Plugin for FlatCore2dPlugin {
    fn build(&self, app: &mut App) {

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<DrawFunctions<PrimitiveQuad>>()
                .add_system_to_stage(RenderStage::Extract, insert_render_phase_for_cameras);

            let mut core_2d_graph = RenderGraph::default();
            core_2d_graph.add_node(
                graph::main::NODE,
                Main2dRenderNode::new(&mut render_app.world),
            );
            let input_node_id = core_2d_graph
                .set_input(vec![SlotInfo::new(graph::input::IN_VIEW, SlotType::Entity)]);
            core_2d_graph
                .add_slot_edge(
                    input_node_id,
                    graph::input::IN_VIEW,
                    graph::main::NODE,
                    Main2dRenderNode::NODE_INPUT_IN_VIEW,
                )
                .unwrap();

            let mut main_render_graph = render_app.world.resource_mut::<RenderGraph>();
            main_render_graph.add_sub_graph(graph::NAME, core_2d_graph);
        }
    }
}

#[derive(Component, Debug)]
pub struct PrimitiveQuad {
    pub sort_key: FloatOrd,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub entity: Entity,
    pub item_range: Range<u32>,
}

impl PhaseItem for PrimitiveQuad {
    type SortKey = FloatOrd;

    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

impl EntityPhaseItem for PrimitiveQuad {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl CachedRenderPipelinePhaseItem for PrimitiveQuad {
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

pub struct Main2dRenderNode {
    query: QueryState<(Read<RenderPhase<PrimitiveQuad>>, Read<ViewTarget>)>,
}

impl Main2dRenderNode {
    pub const NODE_INPUT_IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for Main2dRenderNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::NODE_INPUT_IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        println!("-- MainRenderNode");
        let view_entity = graph.get_input_entity(Self::NODE_INPUT_IN_VIEW)?;
        let query = self.query.get_manual(world, view_entity);

        if let Ok((render_quads, view_target)) = query {
            let render_pass =
                render_context
                    .command_encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(view_target.get_color_attachment(
                            wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                                store: true,
                            },
                        ))],
                        depth_stencil_attachment: None,
                    });
            let mut tracked_pass = TrackedRenderPass::new(render_pass);

            let draw_functions = world.resource::<DrawFunctions<PrimitiveQuad>>();
            let mut draw_functions = draw_functions.write();

            for render_quad in &render_quads.items {
                println!("-- PrimitiveQuad: {:?}", render_quad.entity);
                let id = render_quad.draw_function;
                let draw_function = draw_functions.get_mut(id).expect("Draw Function exists");
                draw_function.draw(world, &mut tracked_pass, view_entity, render_quad);
            }
        }

        Ok(())
    }
}

#[derive(Component, Default)]
pub struct Camera2d;

#[derive(Bundle)]
pub struct Camera2dBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: OrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_2d: Camera2d,
}

impl Default for Camera2dBundle {
    fn default() -> Self {
        Self::new_with_far(1000.0)
    }
}

impl Camera2dBundle {
    /// Create an orthographic projection camera with a custom `Z` position.
    ///
    /// The camera is placed at `Z=far-0.1`, looking toward the world origin `(0,0,0)`.
    /// Its orthographic projection extends from `0.0` to `-far` in camera view space,
    /// corresponding to `Z=far-0.1` (closest to camera) to `Z=-0.1` (furthest away from
    /// camera) in world space.
    pub fn new_with_far(far: f32) -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let projection = OrthographicProjection {
            far,
            ..Default::default()
        };
        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        let view_projection =
            projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            projection.far(),
        );
        Self {
            camera_render_graph: CameraRenderGraph::new(crate::core_2d::graph::NAME),
            projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
            camera: Camera::default(),
            camera_2d: Camera2d::default(),
        }
    }
}

fn insert_render_phase_for_cameras(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &Camera), With<Camera2d>>>,
) {
    for (entity, camera) in cameras.iter() {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(RenderPhase::<PrimitiveQuad>::default());
        }
    }
}
