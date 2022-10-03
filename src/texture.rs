use anyhow::*;
use image::GenericImageView;

use crate::render::resource::bind::{AsBindingSet, Binding, BindingLayoutEntry, IntoBindingSet};

pub enum PixelFormat {
    G8,
    RGBA8,
}

impl PixelFormat {
    pub fn depth(&self) -> u32 {
        match self {
            PixelFormat::G8 => 1,
            PixelFormat::RGBA8 => 4,
        }
    }

    pub fn bytes(&self) -> u32 {
        match self {
            PixelFormat::G8 => 1,
            PixelFormat::RGBA8 => 4,
        }
    }
}

impl From<&PixelFormat> for wgpu::TextureFormat {
    fn from(p: &PixelFormat) -> Self {
        match p {
            PixelFormat::G8 => wgpu::TextureFormat::R8Unorm,
            PixelFormat::RGBA8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        }
    }
}

pub struct RawImage<'a> {
    pub bytes: &'a [u8],
    pub dim: (u32, u32, u32),
    pub pixel_format: PixelFormat,
}

impl<'a> RawImage<'a> {
    pub fn new(bytes: &'a [u8], dim: (u32, u32), pixel_format: PixelFormat) -> Self {
        Self {
            bytes,
            dim: (dim.0, dim.1, pixel_format.depth()),
            pixel_format,
        }
    }

    pub fn bytes_per_row(&self) -> u32 {
        self.pixel_format.bytes() * self.dim.0
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn test_new() -> Self {
        todo!()
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let dim = img.dimensions();
        let raw_img = RawImage::new(&rgba, dim, PixelFormat::RGBA8);
        Self::from_raw_image(device, queue, &raw_img, Some(label))
    }

    pub fn from_raw_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        raw_img: &RawImage,
        label: Option<&str>,
    ) -> Result<Self> {
        // let rgba = img.to_rgba8(); // RGBA Specific
        // let dim = img.dimensions();

        let size = wgpu::Extent3d {
            width: raw_img.dim.0,
            height: raw_img.dim.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: (&raw_img.pixel_format).into(), // wgpu::TextureFormat::Rgba8UnormSrgb, // RGBA Specific
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            raw_img.bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(raw_img.bytes_per_row()), // RGBA Specific
                rows_per_image: std::num::NonZeroU32::new(raw_img.dim.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default() // lod_min_clamp,
                                 // lod_max_clamp,
                                 // compare,
                                 // anisotropy_clamp,
                                 // border_color,
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

impl Binding for wgpu::TextureView {
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        BindingLayoutEntry {
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        wgpu::BindingResource::TextureView(self)
    }
}

impl Binding for wgpu::Sampler {
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        BindingLayoutEntry {
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        wgpu::BindingResource::Sampler(self)
    }
}

impl<'a> AsBindingSet<'a> for Texture {
    type Set = (&'a wgpu::TextureView, &'a wgpu::Sampler);

    fn as_binding_set(&'a self) -> Self::Set {
        (&self.view, &self.sampler)
    }
}
impl<'a> IntoBindingSet for &'a Texture {
    type Set = (&'a wgpu::TextureView, &'a wgpu::Sampler);

    fn into_binding_set(self) -> Self::Set {
        (&self.view, &self.sampler)
    }
}
