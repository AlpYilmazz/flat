use bevy_ecs::{prelude::{Bundle, Component}, world::{FromWorld, World}};
use bytemuck::{Pod, Zeroable};
use cgmath::*;
use repr_trait::C;

use super::resource::uniform::{GpuUniform, StageLockedUniform, Uniform, UpdateGpuUniform};

#[derive(Bundle)]
pub struct PerspectiveCameraBundle {
    pub camera: Camera,
    pub view: CameraView,
    pub proj: PerspectiveProjection,
    pub uniform: Uniform<Camera>,
}

impl FromWorld for PerspectiveCameraBundle {
    fn from_world(world: &mut World) -> Self {
        let device = world.get_resource::<wgpu::Device>().expect("Render device not found in the world");
        Self::new(device)
    }
}

impl PerspectiveCameraBundle {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut camera = Camera::default();
        let camera_view = CameraView::default();
        let perspective_projection = PerspectiveProjection::default();
        camera.view_matrix = camera_view.build_view_matrix();
        camera.projection_matrix = perspective_projection.build_projection_matrix();

        let camera_uniform: Uniform<Camera> = Uniform::new(device, camera.generate_uniform());
        
        Self {
            camera: camera,
            view: camera_view,
            proj: perspective_projection,
            uniform: camera_uniform,
        }
    }
}

#[derive(Component)]
pub struct Camera {
    pub view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
}
impl UpdateGpuUniform for Camera {
    type GU = CameraUniform;

    fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
        gpu_uniform.view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.projection_matrix * self.view_matrix).into();
    }
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::identity(),
        }
    }
}

#[repr(C)]
#[derive(Component, Debug, Clone, Copy, C, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}
impl GpuUniform for CameraUniform {}
impl StageLockedUniform for CameraUniform {
    const FORCE_STAGE: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX;
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
