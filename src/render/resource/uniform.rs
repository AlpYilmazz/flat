use std::marker::PhantomData;

use bevy_ecs::{
    prelude::Component,
    world::{FromWorld, World},
};
use bytemuck::{Pod, Zeroable};
use repr_trait::C;
use wgpu::util::DeviceExt;

use super::bind::{Binding, BindingLayoutEntry};

pub trait GpuUniform: C + Pod + Zeroable + Send + Sync + 'static {
    const STAGE: wgpu::ShaderStages;
}

pub trait HandleGpuUniform {
    type GU: GpuUniform;

    fn generate_uniform(&self) -> Self::GU
    where
        Self::GU: Default,
    {
        let mut gpu_uniform = Self::GU::default();
        self.update_uniform(&mut gpu_uniform);
        gpu_uniform
    }

    fn update_uniform(&self, gpu_uniform: &mut Self::GU);
}

#[derive(Component)]
pub struct Uniform<H>
where
    H: HandleGpuUniform,
{
    pub gpu_uniform: H::GU,
    buffer: UniformBuffer<H::GU>,
    _uniform_repr: PhantomData<H>,
}

impl<H> FromWorld for Uniform<H>
where
    H: HandleGpuUniform,
    H::GU: Default,
{
    fn from_world(world: &mut World) -> Self {
        let device = world
            .get_resource::<wgpu::Device>()
            .expect("Render device not found in the world");
        Self::new_default(device)
    }
}

impl<H> Uniform<H>
where
    H: HandleGpuUniform,
{
    pub fn new_at(device: &wgpu::Device, stage: wgpu::ShaderStages, gpu_uniform: H::GU) -> Self {
        let buffer = UniformBuffer::new_init_at(device, stage, gpu_uniform);
        Self {
            gpu_uniform,
            buffer,
            _uniform_repr: PhantomData,
        }
    }

    pub fn new_default_at(device: &wgpu::Device, stage: wgpu::ShaderStages) -> Self
    where
        H::GU: Default,
    {
        Self::new_at(device, stage, H::GU::default())
    }

    pub fn new(device: &wgpu::Device, gpu_uniform: H::GU) -> Self {
        let buffer = UniformBuffer::new_init(device, gpu_uniform);
        Self {
            gpu_uniform,
            buffer,
            _uniform_repr: PhantomData,
        }
    }

    pub fn new_default(device: &wgpu::Device) -> Self
    where
        H::GU: Default,
    {
        Self::new(device, H::GU::default())
    }

    pub fn sync_buffer(&self, queue: &wgpu::Queue) {
        self.buffer.update(queue, self.gpu_uniform);
    }
}

impl<H> AsRef<Self> for Uniform<H>
where
    H: HandleGpuUniform,
{
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<H> Binding for Uniform<H>
where
    H: HandleGpuUniform,
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

    pub fn new_init(device: &wgpu::Device, init: T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[init]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            stage: T::STAGE,
            buffer,
            _marker: PhantomData,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, val: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[val]));
    }
}

impl<T: GpuUniform> AsRef<Self> for UniformBuffer<T> {
    fn as_ref(&self) -> &Self {
        self
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
