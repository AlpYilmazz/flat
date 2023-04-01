use bevy::{
    prelude::{Assets, Component, Handle, Query, ResMut},
    reflect::TypeUuid,
};

use crate::render::{
    resource::renderer::{RenderDevice, RenderQueue},
    RenderAsset,
};

use super::{GpuTexture, Image, ImageDim};

#[derive(TypeUuid)]
#[uuid = "8E7C2F0A-6BB8-485C-917E-6B605A0DDF29"]
pub struct ImageArray {
    data: Vec<u8>,
    pub dim: ImageDim,
    pub count: u32,
}

// TODO: Can be implemented with dependencies system
// #[derive(Default)]
// pub struct ImageArrLoader;
// impl AssetLoader for ImageArrLoader {}

impl ImageArray {
    pub fn new(dim: ImageDim) -> Self {
        Self {
            data: Vec::new(),
            dim,
            count: 0,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn add(&mut self, data: &[u8], dim: ImageDim) {
        debug_assert_eq!(dim, self.dim);
        // NOTE: may find a more performant solution
        self.data.extend_from_slice(data);
        self.count += 1;
    }

    pub fn overwrite(&mut self, pos: u32, data: &[u8], dim: ImageDim) {
        let byte_count = dim.total_bytes().min(self.dim.total_bytes()).min(data.len() as u32);
        let data = &data[..byte_count as usize];
        
        let start = pos * self.dim.total_bytes();
        let end = start + byte_count;
        let Some(self_slice) = self.data.get_mut(start as usize .. end as usize) else {
            return;
        };
        self_slice.copy_from_slice(data);
    }

    pub fn from_images(mut images: impl Iterator<Item = Image>) -> Self {
        let Some(img0) = images.next() else {
            panic!("Cannot create ImageArray from empty iterator");
        };
        let mut img_array = Self::new(img0.dim());
        img_array.add(&img0.img.to_rgba8(), img0.dim());

        for img in images {
            img_array.add(&img.img.to_rgba8(), img.dim());
        }

        img_array
    }
}

impl FromIterator<Image> for ImageArray {
    fn from_iter<T: IntoIterator<Item = Image>>(images: T) -> Self {
        Self::from_images(images.into_iter())
    }
}

impl RenderAsset for ImageArray {
    type PreparedAsset = GpuTexture;

    fn prepare(&self, device: &RenderDevice, queue: &RenderQueue) -> Option<Self::PreparedAsset> {
        match GpuTexture::create_texture_array(device, queue, &self.data, self.dim, self.count) {
            Ok(e) => Some(e),
            Err(err) => {
                dbg!(err);
                None
            }
        }
    }
}

#[derive(Component, Default)]
pub struct ImageArrayHandle {
    pub image_arr: Option<Handle<ImageArray>>,
    pub images: Vec<Handle<Image>>,
}

impl ImageArrayHandle {
    /// NOTE: Can images be empty?
    pub fn with_images(images: Vec<Handle<Image>>) -> Self {
        Self {
            image_arr: None,
            images,
        }
    }
}

pub fn create_image_arr_from_images(
    mut image_assets: ResMut<Assets<Image>>,
    mut image_arr_assets: ResMut<Assets<ImageArray>>,
    mut query: Query<&mut ImageArrayHandle>,
) {
    for mut image_arr in query.iter_mut() {
        if image_arr.images.is_empty() {
            continue;
        }

        let all_ready = image_arr
            .images
            .iter()
            .all(|handle| image_assets.contains(handle));

        if all_ready {
            let image_array = image_arr
                .images
                .iter()
                .map(|handle| (image_assets.remove(handle).unwrap()))
                .collect();

            image_arr.image_arr = Some(image_arr_assets.add(image_array));
            image_arr.images.clear();
            println!("ImageArray created");
        }
    }
}
