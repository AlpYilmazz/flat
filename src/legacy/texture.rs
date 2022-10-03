use std::{fs::File, io::BufReader, convert::TryInto, path::Path};

use image::GenericImageView;
use anyhow::*;

use crate::resource::buffer::BindGroup;

pub enum PixelFormat {
    G8, RGBA8
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
    const ENTRIES: &'static [wgpu::BindGroupLayoutEntry] = 
        &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }
        ];

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

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: (&raw_img.pixel_format).into(), // wgpu::TextureFormat::Rgba8UnormSrgb, // RGBA Specific
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            }
        );

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
            size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                // label,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
                // lod_min_clamp,
                // lod_max_clamp,
                // compare,
                // anisotropy_clamp,
                // border_color,
            }
        );

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
        label: &str
    ) -> Self {
        let size = wgpu::Extent3d { // 2.
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
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
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
            }
        );

        Self { texture, view, sampler }
    }
}

impl BindGroup for Texture {
    fn layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &Self::ENTRIES,
        }
    }
}


fn open_img(path: impl AsRef<Path>) -> image::DynamicImage {
    let file = File::open(path).unwrap();
    let img = image::load(BufReader::new(file), image::ImageFormat::Jpeg).unwrap();

    img
}


pub struct TextureArray<const N: usize> {
    // pub textures: [wgpu::Texture; N],
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl<const N: usize> TextureArray<N> {
    const ENTRIES: &'static [wgpu::BindGroupLayoutEntry] = 
        &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: std::num::NonZeroU32::new(N as u32),
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }
        ];

    // pub fn from_bytes(
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     bytes: &[u8],
    // ) -> Self {

    // }

    pub fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        dir: &str,
        names: [&str; N],
        ext: &str,
    ) -> Result<Self> {
        assert_ne!(N, 0);

        let imgs = names.iter()
        .map(|name| {
                let img = open_img(format!("{}/{}.{}", dir, name, ext));
                (img.dimensions(), img.to_rgba8())
            })
            .collect::<Vec<_>>();
        
        let raw_imgs = imgs.iter()
            .map(|(dim, rgba)| {
                RawImage::new(&rgba, *dim, PixelFormat::RGBA8)
            })
            .collect::<Vec<_>>();

        let bytes = raw_imgs.iter()
        .flat_map(|r| r.bytes)
        .map(|a| *a)
        .collect::<Vec<_>>();
        
        let raw_img_0 = &raw_imgs[0];

        let texture_array_size = wgpu::Extent3d {
            width: raw_img_0.dim.0,
            height: raw_img_0.dim.1,
            depth_or_array_layers: N as u32,
        };
        dbg!(texture_array_size);

        // TODO: `texture` is going to hold all textures
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: None,
                size: texture_array_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: (&raw_img_0.pixel_format).into(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            }
        );
        
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(raw_img_0.bytes_per_row()), // RGBA Specific
                rows_per_image: std::num::NonZeroU32::new(raw_img_0.dim.1),
            },
            texture_array_size
        );        

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(N as u32),
            ..Default::default()
        });

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                // label,
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
                // lod_min_clamp,
                // lod_max_clamp,
                // compare,
                // anisotropy_clamp,
                // border_color,
            }
        );

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}

impl<const N: usize> BindGroup for TextureArray<N> {
    fn layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &Self::ENTRIES,
        }
    }
}