use bevy::{
    prelude::*,
    render::{
        render_graph::{RenderGraph, SlotInfo, SlotType, Node, RenderGraphContext, NodeRunError},
        render_phase::{DrawFunctionId, DrawFunctions, PhaseItem, EntityPhaseItem, CachedRenderPipelinePhaseItem, RenderPhase, TrackedRenderPass},
        render_resource::CachedRenderPipelineId,
        RenderApp, view::ViewTarget, renderer::RenderContext,
    }, ecs::system::lifetimeless::Read,
};
use float_ord::FloatOrd;

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
        app.init_resource::<DrawFunctions<PrimitiveQuad>>();

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
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

#[derive(Component)]
pub struct PrimitiveQuad {
    pub sort_key: FloatOrd<f32>,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub entity: Entity,
}

impl PhaseItem for PrimitiveQuad {
    type SortKey = FloatOrd<f32>;

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
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        ))],
                        depth_stencil_attachment: None,
                    });
            let mut render_pass = TrackedRenderPass::new(render_pass);

            let draw_functions = world.resource::<DrawFunctions<PrimitiveQuad>>();
            let mut draw_functions = draw_functions.write();

            for render_quad in &render_quads.items {
                let id = render_quad.draw_function;
                let draw_function = draw_functions.get_mut(id).expect("Draw Function exists");
                draw_function.draw(world, &mut render_pass, view_entity, render_quad);
            }
        }

        Ok(())
    }
}
