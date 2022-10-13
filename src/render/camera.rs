use std::collections::HashMap;

use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::{Bundle, Component, Entity},
    query::{Added, Changed, Or},
    schedule::ParallelSystemDescriptorCoercion,
    system::{Query, RemovedComponents, Res, ResMut},
};
use bytemuck::{Pod, Zeroable};
use cgmath::*;
use repr_trait::C;

use crate::{
    render::resource::uniform::UniformDesc,
    util::{store, Refer, Sink, Store},
    window::WindowId,
};

use super::{
    resource::{
        bind::BindingSet,
        uniform::{GpuUniform, HandleGpuUniform, Uniform},
    },
    RenderStage, SurfaceReconfigure,
};

pub struct FlatCameraPlugin;
impl Plugin for FlatCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtractedCameras>()
            .add_system_to_stage(
                RenderStage::Prepare,
                reconfigure_cameras.after(SurfaceReconfigure),
            )
            .add_system_to_stage(RenderStage::Extract, extract_cameras);
    }
}

pub struct ExtractedCamera {
    pub render_window: WindowId,
    pub uniform: Uniform<Camera>,
    pub bind_refer: Refer<wgpu::BindGroup>,
}
pub type ExtractedCameras = HashMap<Entity, ExtractedCamera>;

fn extract_cameras(
    device: Res<wgpu::Device>,
    queue: Res<wgpu::Queue>,
    query: Query<
        (Entity, &Camera, Added<Camera>, Changed<Camera>),
        Or<(Added<Camera>, Changed<Camera>)>,
    >,
    removed: RemovedComponents<Camera>,
    mut extracted_cameras: ResMut<ExtractedCameras>,
    mut bind_groups: ResMut<Store<wgpu::BindGroup>>,
) {
    for (entity, camera, added, changed) in query.iter() {
        if added {
            assert!(!extracted_cameras.contains_key(&entity));

            let uniform = Uniform::new(&device, camera.generate_uniform());
            let bind_group = uniform
                .as_ref()
                .into_bind_group(&device, &UniformDesc::default());
            let bind_refer = store(&mut bind_groups, bind_group);
            extracted_cameras.insert(
                entity,
                ExtractedCamera {
                    render_window: camera.render_window,
                    uniform,
                    bind_refer,
                },
            );
        } else if changed {
            let ExtractedCamera { uniform, .. } = extracted_cameras.get_mut(&entity).unwrap();
            camera.update_uniform(&mut uniform.gpu_uniform);
            uniform.sync_buffer(&queue);
        };
    }

    removed
        .iter()
        .for_each(|entity| extracted_cameras.remove(&entity).sink());
}

pub fn reconfigure_cameras(
    mut cameras: Query<
        (
            &mut Camera,
            &CameraView,
            &PerspectiveProjection,
            Changed<CameraView>,
            Changed<PerspectiveProjection>,
        ),
        Or<(Changed<CameraView>, Changed<PerspectiveProjection>)>,
    >,
) {
    for (mut camera, view, pers_proj, view_changed, pers_changed) in cameras.iter_mut() {
        if view_changed {
            camera.view_matrix = view.build_view_matrix();
        }
        if pers_changed {
            camera.projection_matrix = pers_proj.build_projection_matrix();
        }
    }
}

#[derive(Bundle)]
pub struct PerspectiveCameraBundle {
    pub camera: Camera,
    pub view: CameraView,
    pub proj: PerspectiveProjection,
}

impl Default for PerspectiveCameraBundle {
    fn default() -> Self {
        Self::new(WindowId::primary())
    }
}

impl PerspectiveCameraBundle {
    pub fn new(render_window: WindowId) -> Self {
        let camera_view = CameraView::default();
        let perspective_projection = PerspectiveProjection::default();
        let camera = Camera {
            render_window,
            view_matrix: camera_view.build_view_matrix(),
            projection_matrix: perspective_projection.build_projection_matrix(),
        };

        Self {
            camera,
            view: camera_view,
            proj: perspective_projection,
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Visible;

#[derive(Component)]
pub struct Camera {
    pub render_window: WindowId,
    pub view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
}
impl HandleGpuUniform for Camera {
    type GU = CameraUniform;

    fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
        gpu_uniform.view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.projection_matrix * self.view_matrix).into();
    }
}

#[repr(C)]
#[derive(Component, Debug, Clone, Copy, C, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}
impl GpuUniform for CameraUniform {
    const STAGE: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX;
}
impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }
}

#[derive(Component)]
pub struct CameraView {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
}

impl CameraView {
    pub fn build_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: Vector3::unit_y(),
        }
    }
}

#[derive(Component)]
pub struct PerspectiveProjection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl PerspectiveProjection {
    pub fn build_projection_matrix(&self) -> Matrix4<f32> {
        cgmath::perspective(Rad(self.fovy), self.aspect, self.znear, self.zfar)
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            aspect: 1.0,
            fovy: std::f32::consts::PI / 4.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
