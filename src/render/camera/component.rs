use bevy::{
    prelude::{Bundle, Component, Entity, GlobalTransform, Handle, Transform, Mat4},
    window::WindowId,
};
use encase::ShaderType;

use crate::render::{texture::Image, view::window::PreparedWindows, RenderAssets, resource::uniform::HandleGpuUniform};

#[derive(Bundle, Default)]
pub struct CameraBundle<P: Projection> {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera: Camera,
    pub projection: P,
    pub visible_entities: VisibleEntities,
    // pub render_layers: RenderLayers,
}

pub enum RenderTarget {
    Image(Handle<Image>),
    Window(WindowId),
}

impl RenderTarget {
    pub fn get_window(&self) -> Option<WindowId> {
        if let RenderTarget::Window(id) = self {
            return Some(*id);
        }
        return None;
    }

    pub fn holds_image(&self, image_handle: Handle<Image>) -> bool {
        match self {
            RenderTarget::Image(handle) => image_handle.eq(handle),
            RenderTarget::Window(_) => false,
        }
    }

    pub fn holds_window(&self, window_id: WindowId) -> bool {
        match self {
            RenderTarget::Image(_) => false,
            RenderTarget::Window(id) => window_id.eq(id),
        }
    }

    pub fn get_view<'a>(
        &self,
        gpu_textures: &'a RenderAssets<Image>,
        windows: &'a PreparedWindows,
    ) -> &'a wgpu::TextureView {
        match self {
            RenderTarget::Image(handle) => &gpu_textures.get(&handle.id()).unwrap().view,
            RenderTarget::Window(id) => {
                &windows
                    .get(id)
                    .unwrap()
                    .surface_texture
                    .as_ref()
                    .unwrap()
                    .view
            }
        }
    }
}

pub struct CameraMatrices {
    pub view: Mat4,
    pub proj: Mat4,
}

impl CameraMatrices {
    fn identity() -> Self {
        Self {
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
        }
    }
}

#[derive(Component)]
pub struct Camera {
    pub render_target: RenderTarget,
    pub computed: CameraMatrices,
    pub is_active: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            render_target: RenderTarget::Window(WindowId::primary()),
            computed: CameraMatrices::identity(),
            is_active: true,
        }
    }
}

pub trait Projection: Component {
    fn update(&mut self, width: f32, height: f32);
    fn build_projection_matrix(&self) -> Mat4;
}

#[derive(Component)]
pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Projection for OrthographicProjection {
    fn update(&mut self, width: f32, height: f32) {
        println!("{} {}", width, height);
        todo!()
    }

    fn build_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
}

#[derive(Component)]
pub struct PerspectiveProjection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            aspect: 1.0,
            fovy: std::f32::consts::PI / 4.0,
            zfar: 1000.0,
            znear: 0.1,
        }
    }
}

impl Projection for PerspectiveProjection {
    fn update(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    fn build_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Component)]
pub struct Visible;

#[derive(Component, Default)]
pub struct VisibleEntities {
    pub(super) entities: Vec<Entity>,
}

impl VisibleEntities {
    pub fn iter(&self) -> std::slice::Iter<Entity> {
        self.entities.iter()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

pub type LayerMask = u32; // 32 layers
pub type Layer = u8; // In runtime range of 0..31
const DEFAULT_LAYER: Layer = 1;
const DEFAULT_LAYER_MASK: LayerMask = 1 << DEFAULT_LAYER;

#[derive(Component)]
pub struct RenderLayers(LayerMask);

impl Default for RenderLayers {
    fn default() -> Self {
        Self(DEFAULT_LAYER_MASK)
    }
}

impl RenderLayers {
    pub const NUM_LAYERS: usize = std::mem::size_of::<LayerMask>() * 8;

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn with(&mut self, layer: Layer) -> &mut Self {
        assert!((layer as usize) < Self::NUM_LAYERS);
        self.0 |= 1 << layer;
        self
    }

    pub fn without(&mut self, layer: Layer) -> &mut Self {
        assert!((layer as usize) < Self::NUM_LAYERS);
        self.0 &= reverse_bits(1 << layer);
        self
    }

    pub fn with_layers(&mut self, layers: &[Layer]) -> &mut Self {
        for layer in layers {
            self.with(*layer);
        }
        self
    }
    // with_layers and without_layers different types of implementations
    pub fn without_layers(&mut self, layers: &[Layer]) -> &mut Self {
        let combined = layers.iter().fold(0u8, |acc, layer| acc | (1 << *layer));
        self.without(combined);
        self
    }

    pub fn intersects(&self, other: &Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn contains(&self, layer: Layer) -> bool {
        assert!((layer as usize) < Self::NUM_LAYERS);
        (self.0 & (1 << layer)) != 0
    }
}

fn reverse_bits(a: u32) -> u32 {
    a ^ u32::MAX
}

pub fn layers_intersect(layers1: Option<&RenderLayers>, layers2: Option<&RenderLayers>) -> bool {
    match (layers1, layers2) {
        (None, None) => true,
        (None, Some(l2)) => l2.contains(DEFAULT_LAYER),
        (Some(l1), None) => l1.contains(DEFAULT_LAYER),
        (Some(l1), Some(l2)) => l1.intersects(l2),
    }
}

#[derive(Clone, ShaderType)]
pub struct CameraUniforms {
    view_proj: Mat4,
    view: Mat4,
    proj: Mat4,
}

impl HandleGpuUniform for Camera {
    type GU = CameraUniforms;

    fn into_uniform(&self) -> Self::GU {
        CameraUniforms {
            view_proj: self.computed.view * self.computed.proj,
            view: self.computed.view,
            proj: self.computed.proj,
        }
    }
}