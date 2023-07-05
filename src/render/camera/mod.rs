use bevy::{
    prelude::{
        CoreStage, Entity, EventReader, GlobalTransform, IntoSystemDescriptor, Plugin, Query,
        SystemLabel, With,
    },
    window::{ModifiesWindows, WindowResized},
};

use self::component::*;

use super::resource::component_uniform::AddComponentUniform;

pub mod component;

pub struct FlatCameraPlugin;
impl Plugin for FlatCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_projection_systems::<OrthographicProjection>()
            .add_projection_systems::<PerspectiveProjection>()
            .add_component_uniform::<Camera>()
            .add_system_to_stage(CoreStage::PostUpdate, visibility_system);
    }
}

#[derive(SystemLabel)]
pub struct ProjectionUpdate;

trait AddProjectionSystems {
    fn add_projection_systems<P: Projection>(&mut self) -> &mut Self;
}
impl AddProjectionSystems for bevy::prelude::App {
    fn add_projection_systems<P: Projection>(&mut self) -> &mut Self {
        self.add_system_to_stage(
            CoreStage::PostUpdate,
            update_projections_on_window_resize::<P>
                .label(ProjectionUpdate)
                .after(ModifiesWindows),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            update_camera_values::<P>.after(ProjectionUpdate),
        )
    }
}

pub fn update_projections_on_window_resize<P: Projection>(
    mut events: EventReader<WindowResized>,
    mut query: Query<(&Camera, &mut P)>,
) {
    for WindowResized {
        id: window_id,
        width,
        height,
    } in events.iter()
    {
        if *width <= 0.0 || *height <= 0.0 {
            continue;
        }
        for (camera, mut proj) in query.iter_mut() {
            if camera.render_target.holds_window(*window_id) {
                proj.update(*width, *height);
            }
        }
    }
}

pub fn update_camera_values<P: Projection>(mut query: Query<(&mut Camera, &GlobalTransform, &P)>) {
    for (mut camera, transform, proj) in query.iter_mut() {
        camera.computed.view = transform.compute_matrix();
        camera.computed.proj = proj.build_projection_matrix();
    }
}

pub fn visibility_system(
    entities: Query<(Entity, &Visibility, Option<&RenderLayers>)>,
    mut cameras: Query<(Option<&RenderLayers>, &mut VisibleEntities), With<Camera>>,
) {
    for (_, mut visible_entities) in cameras.iter_mut() {
        visible_entities.clear();
    }
    
    for (entity, visibility, entity_layers) in entities.iter() {
        if !visibility.visible { continue; }
        for (camera_layers, mut visible_entities) in cameras.iter_mut() {
            if layers_intersect(entity_layers, camera_layers) {
                visible_entities.entities.push(entity);
            }
        }
    }
}
