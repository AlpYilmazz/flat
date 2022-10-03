use std::{marker::PhantomData, num::NonZeroU32};

use bytemuck::{Pod, Zeroable};
use repr_trait::C;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct BindingLayoutEntry {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<NonZeroU32>,
}

#[derive(Debug)]
pub struct BindingSetLayoutDescriptor {
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindingLayoutEntry {
    pub fn with_binding(self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: self.ty,
            count: self.count,
        }
    }
}

pub trait Binding {
    fn get_layout_entry(&self) -> BindingLayoutEntry;
    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a>;
}

pub trait BindingSet {
    fn layout_desc(&self) -> BindingSetLayoutDescriptor;
    fn into_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
}

pub trait AsBindingSet<'a> {
    type Set: BindingSet;

    fn as_binding_set(&'a self) -> Self::Set;
}
pub trait IntoBindingSet {
    type Set: BindingSet;

    fn into_binding_set(self) -> Self::Set;
}
impl<T: BindingSet> IntoBindingSet for T {
    type Set = T;

    fn into_binding_set(self) -> Self::Set {
        self
    }
}

#[allow(non_snake_case)]
impl<B0> BindingSet for &B0
where
    B0: Binding,
{
    fn layout_desc(&self) -> BindingSetLayoutDescriptor {
        BindingSetLayoutDescriptor {
            entries: vec![self.get_layout_entry().with_binding(0)],
        }
    }

    fn into_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let bs_layout = self.layout_desc();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &bs_layout.entries,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.get_resource(),
            }],
        });

        bind_group
    }
}

pub trait GpuUniform: C + Pod + Zeroable {}

pub trait StageLockedUniform: GpuUniform {
    const FORCE_STAGE: wgpu::ShaderStages;
}

pub trait UpdateGpuUniform {
    type GU: GpuUniform;

    fn update_uniform(&self, gpu_uniform: &mut Self::GU);
}

pub struct Uniform<H>
where
    H: UpdateGpuUniform,
{
    pub gpu_uniform: H::GU,
    buffer: UniformBuffer<H::GU>,
    _uniform_repr: PhantomData<H>,
}

impl<H> Uniform<H>
where
    H: UpdateGpuUniform,
{
    pub fn new(device: &wgpu::Device, stage: wgpu::ShaderStages, gpu_uniform: H::GU) -> Self {
        let buffer = UniformBuffer::new_init_at(device, stage, gpu_uniform);
        Self {
            gpu_uniform,
            buffer,
            _uniform_repr: PhantomData,
        }
    }

    pub fn sync_buffer(&self, queue: &wgpu::Queue) {
        self.buffer.update(queue, self.gpu_uniform);
    }
}

impl<H> Uniform<H>
where
    H: UpdateGpuUniform,
    H::GU: Default,
{
    pub fn new_default(device: &wgpu::Device, stage: wgpu::ShaderStages) -> Self {
        Self::new(device, stage, H::GU::default())
    }
}

impl<H> Binding for Uniform<H>
where
    H: UpdateGpuUniform,
{
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        self.buffer.get_layout_entry()
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        self.buffer.get_resource()
    }
}

pub struct UniformBuffer<T: GpuUniform> {
    stage: wgpu::ShaderStages,
    buffer: wgpu::Buffer,
    _marker: PhantomData<T>,
}

impl<T: GpuUniform> UniformBuffer<T> {
    pub fn new_init_at(device: &wgpu::Device, stage: wgpu::ShaderStages, init: T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[init]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            stage,
            buffer,
            _marker: PhantomData,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, val: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[val]));
    }
}

impl<T: StageLockedUniform> UniformBuffer<T> {
    pub fn new_init(device: &wgpu::Device, init: T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[init]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            stage: T::FORCE_STAGE,
            buffer,
            _marker: PhantomData,
        }
    }
}

impl<T: GpuUniform> Binding for UniformBuffer<T> {
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        BindingLayoutEntry {
            visibility: self.stage,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        self.buffer.as_entire_binding()
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use cgmath::*;

    use crate::texture::Texture;

    use super::*;

    pub struct Camera {
        pub view_matrix: Matrix4<f32>,
        pub projection_matrix: Matrix4<f32>,
    }
    impl UpdateGpuUniform for Camera {
        type GU = CameraUniform;

        fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
            gpu_uniform.view_proj = (self.projection_matrix * self.view_matrix).into();
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
    #[derive(Debug, Clone, Copy, C, Pod, Zeroable)]
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

    pub struct Transform {
        pub translation: Vector3<f32>,
        pub scale: Vector3<f32>,
        pub rotation: Quaternion<f32>,
    }
    impl UpdateGpuUniform for Transform {
        type GU = ModelUniform;

        fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
            gpu_uniform.model = (Matrix4::from_translation(self.translation)
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
                * Matrix4::from(self.rotation))
            .into();
        }
    }
    impl Default for Transform {
        fn default() -> Self {
            Self {
                translation: Vector3::zero(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                rotation: Quaternion::zero(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, C, Pod, Zeroable)]
    pub struct ModelUniform {
        pub model: [[f32; 4]; 4],
    }
    impl GpuUniform for ModelUniform {}
    impl StageLockedUniform for ModelUniform {
        const FORCE_STAGE: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX;
    }
    impl Default for ModelUniform {
        fn default() -> Self {
            Self {
                model: Matrix4::identity().into(),
            }
        }
    }

    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }
    impl Color {
        pub fn from_tuple((r, g, b, a): (f32, f32, f32, f32)) -> Self {
            Self { r, g, b, a }
        }

        pub fn as_tuple(&self) -> (f32, f32, f32, f32) {
            (self.r, self.g, self.b, self.a)
        }
    }
    impl UpdateGpuUniform for Color {
        type GU = ColorUniform;

        fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
            gpu_uniform.color = [self.r, self.g, self.b, self.a];
        }
    }
    impl Default for Color {
        fn default() -> Self {
            Self::from_tuple((0.0, 0.0, 0.0, 1.0))
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, C, Pod, Zeroable)]
    pub struct ColorUniform {
        pub color: [f32; 4],
    }
    impl GpuUniform for ColorUniform {}
    impl Default for ColorUniform {
        fn default() -> Self {
            Self {
                color: [0.0, 0.0, 0.0, 1.0],
            }
        }
    }

    fn uniform_usage(device: &wgpu::Device, queue: &wgpu::Queue) {
        // Create high level reprs of uniforms
        let camera = Camera::default();
        let transform = Transform::default();
        let color = Color::from_tuple((0.5, 0.5, 0.0, 1.0));

        // Create uniforms
        let mut camera_uniform: Uniform<Camera> =
            Uniform::new_default(device, wgpu::ShaderStages::VERTEX);
        let mut model_transform_uniform: Uniform<Transform> =
            Uniform::new_default(device, wgpu::ShaderStages::VERTEX);
        let mut color_uniform: Uniform<Color> =
            Uniform::new_default(device, wgpu::ShaderStages::FRAGMENT);

        // Update uniforms
        camera.update_uniform(&mut camera_uniform.gpu_uniform);
        transform.update_uniform(&mut model_transform_uniform.gpu_uniform);
        color.update_uniform(&mut color_uniform.gpu_uniform);

        // Sync Gpu buffers with uniform updates
        camera_uniform.sync_buffer(queue);
        model_transform_uniform.sync_buffer(queue);
        color_uniform.sync_buffer(queue);

        // Create BindingSet
        let mvp_binding_set = (&camera_uniform, &model_transform_uniform);
        let color_binding_set = &color_uniform;
        let texture = Texture::test_new();

        // BindingSet into BindGroup
        let mvp_layout_debug = mvp_binding_set.layout_desc();
        let mvp_bind_group = mvp_binding_set.into_bind_group(device);
        let color_bind_group = color_binding_set.into_bind_group(device);
        let texture_bind_group = texture.into_binding_set().into_bind_group(device);
        // texture
        // .as_binding_set()

        // Debug
        dbg!(mvp_layout_debug);
        dbg!(mvp_bind_group);
        dbg!(color_bind_group);
        dbg!(texture_bind_group);
    }
}

macro_rules! impl_binding_set_tuple {
    ($(($ind: literal,$param: ident)),*) => {
        #[allow(non_snake_case)]
        impl<$($param: Binding),*> BindingSet for ($(&$param,)*) {
            fn layout_desc(&self) -> BindingSetLayoutDescriptor {
                let ($($param,)*) = *self;

                BindingSetLayoutDescriptor {
                    entries: vec![
                        $(
                            $param.get_layout_entry().with_binding($ind),
                        )*
                    ],
                }
            }

            fn into_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
                let ($($param,)*) = *self;

                let bs_layout = self.layout_desc();

                let bind_group_layout = device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &bs_layout.entries,
                    }
                );

                let bind_group = device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &bind_group_layout,
                        entries: &[
                            $(
                                wgpu::BindGroupEntry {
                                    binding: $ind,
                                    resource: $param.get_resource(),
                                },
                            )*
                        ],
                    }
                );

                bind_group
            }
        }
    };
}

impl_binding_set_tuple!((0, B0));
impl_binding_set_tuple!((0, B0), (1, B1));
impl_binding_set_tuple!((0, B0), (1, B1), (2, B2));
impl_binding_set_tuple!((0, B0), (1, B1), (2, B2), (3, B3));
impl_binding_set_tuple!((0, B0), (1, B1), (2, B2), (3, B3), (4, B4));
impl_binding_set_tuple!((0, B0), (1, B1), (2, B2), (3, B3), (4, B4), (5, B5));

// #[allow(non_snake_case)]
// impl<B0, B1> BindingSet for (&B0, &B1,)
// where
//     B0: Binding,
//     B1: Binding,
// {
//     fn layout_desc(&self) -> BindingSetLayoutDescriptor {
//         let (B0, B1,) = *self;

//         BindingSetLayoutDescriptor {
//             entries: vec![
//                 B0.get_layout_entry().with_binding(0),
//                 B1.get_layout_entry().with_binding(1),
//             ],
//         }
//     }

//     fn into_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
//         let (B0, B1,) = *self;

//         let bs_layout = self.layout_desc();

//         let bind_group_layout = device.create_bind_group_layout(
//             &wgpu::BindGroupLayoutDescriptor {
//                 label: None,
//                 entries: &bs_layout.entries,
//             }
//         );

//         let bind_group = device.create_bind_group(
//             &wgpu::BindGroupDescriptor {
//                 label: None,
//                 layout: &bind_group_layout,
//                 entries: &[
//                     wgpu::BindGroupEntry {
//                         binding: 0,
//                         resource: B0.get_resource(),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: 1,
//                         resource: B1.get_resource(),
//                     }
//                 ],
//             }
//         );

//         bind_group
//     }
// }
