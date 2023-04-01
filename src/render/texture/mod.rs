use anyhow::*;
use bevy::asset::{AssetLoader, LoadedAsset};
use bevy::prelude::{Deref, DerefMut, Resource};
use bevy::reflect::TypeUuid;
use bevy::utils::HashMap;
use image::{DynamicImage, GenericImageView};

use super::{camera, RenderAsset, RenderDevice, RenderQueue};

pub mod texture_arr;

#[derive(TypeUuid)]
#[uuid = "3F897E85-62CE-4B2C-A957-FCF0CCE649FD"]
pub struct Image {
    pub img: DynamicImage,
    pub prepare: bool,
}

impl Image {
    pub fn dim(&self) -> ImageDim {
        let dimensions = self.img.dimensions();
        ImageDim {
            width: dimensions.0,
            heigth: dimensions.1,
            pixel: PixelFormat::RGBA8, // TODO: extend support
        }
    }
}

#[derive(Default)]
pub struct ImageLoader;
impl AssetLoader for ImageLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async {
            let img = image::load_from_memory(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(Image { img, prepare: true }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg"]
    }
}

#[derive(Default)]
pub struct ImageJustLoader;
impl AssetLoader for ImageJustLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async {
            let img = image::load_from_memory(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(Image {
                img,
                prepare: false,
            }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["just.png", "just.jpg", "just.jpeg"]
    }
}

impl RenderAsset for Image {
    type PreparedAsset = GpuTexture;

    fn prepare(&self, device: &RenderDevice, queue: &RenderQueue) -> Option<Self::PreparedAsset> {
        if !self.prepare {
            return None;
        }

        let rgba = self.img.to_rgba8(); // TODO: extend support
        let dim = self.img.dimensions();
        let raw_img = RawImage::new(&rgba, dim, PixelFormat::RGBA8); // TODO: extend support
        Some(GpuTexture::from_raw_image(device, queue, &raw_img, None).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageDim {
    pub width: u32,
    pub heigth: u32,
    pub pixel: PixelFormat,
    // pub px: u32,
    // pub stride: u32,
}

impl ImageDim {
    pub fn bytes_per_row(&self) -> u32 {
        self.pixel.bytes() * self.width
    }

    pub fn total_bytes(&self) -> u32 {
        self.heigth * self.bytes_per_row()
    }
}

pub struct RawImage<'a> {
    pub bytes: &'a [u8],
    pub dim: (u32, u32, u32), // TODO: refactor as ImageDim
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

pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl GpuTexture {
    pub fn unimplemented_new() -> Self {
        unimplemented!()
    }

    pub fn from_bytes(
        device: &RenderDevice,
        queue: &RenderQueue,
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
        device: &RenderDevice,
        queue: &RenderQueue,
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

    pub fn create_texture_array(
        device: &RenderDevice,
        queue: &RenderQueue,
        data: &[u8],
        dim: ImageDim,
        count: u32,
    ) -> Result<Self> {
        let size = wgpu::Extent3d {
            width: dim.width,
            height: dim.heigth,
            depth_or_array_layers: count,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: (&dim.pixel).into(), // wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(dim.bytes_per_row()),
                rows_per_image: std::num::NonZeroU32::new(dim.heigth),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(count),
            ..Default::default()
        });
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

        // let a = &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                 view_dimension: wgpu::TextureViewDimension::D2Array,
        //                 multisampled: false,
        //             },
        //             count: std::num::NonZeroU32::new(6 /*N as u32*/),
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //             count: None,
        //         }
        //     ];
    }

    pub fn create_depth_texture(
        render_device: &RenderDevice,
        config: &wgpu::SurfaceConfiguration,
        label: Option<&str>,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = render_device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = render_device.create_sampler(&wgpu::SamplerDescriptor {
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

#[derive(Deref)]
pub struct DepthTexture(pub GpuTexture);

impl DepthTexture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub fn create(render_device: &RenderDevice, config: &wgpu::SurfaceConfiguration) -> Self {
        Self(GpuTexture::create_depth_texture(render_device, config, None, Self::DEPTH_FORMAT))
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct DepthTextures(pub HashMap<camera::component::RenderTarget, DepthTexture>);
