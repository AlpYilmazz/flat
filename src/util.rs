use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy_asset::HandleId;
use bevy_ecs::prelude::Component;
use bluenoise::BlueNoise;
use rand_pcg::Pcg64Mcg;

pub struct Store<T> {
    ind: usize,
    pub inner: HashMap<usize, T>,
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self {
            ind: 1,
            inner: Default::default(),
        }
    }
}

impl<T> Store<T> {
    const PRIMARY_ID: usize = 0;
    
    pub fn insert(&mut self, val: T) -> usize {
        self.inner.insert(self.ind, val);
        self.ind += 1;

        self.ind - 1
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        self.inner.get(&key)
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        self.inner.get_mut(&key)
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        self.inner.remove(&key)
    }

    pub fn insert_primary(&mut self, val: T) -> usize {
        self.inner.insert(Self::PRIMARY_ID, val);
        Self::PRIMARY_ID
    }

    pub fn get_primary(&self) -> Option<&T> {
        self.inner.get(&Self::PRIMARY_ID)
    }

    pub fn get_primary_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut(&Self::PRIMARY_ID)
    }

    pub fn remove_primary(&mut self) -> Option<T> {
        self.inner.remove(&Self::PRIMARY_ID)
    }
}

pub struct AssetStore<T>(pub HashMap<HandleId, T>);
impl<T> Default for AssetStore<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T> Deref for AssetStore<T> {
    type Target = HashMap<HandleId, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for AssetStore<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct Refer<T>(usize, PhantomData<fn() -> T>);
impl<T> Refer<T> {
    pub fn to(ind: usize) -> Self {
        Self(ind, PhantomData)
    }
}
impl<T> Deref for Refer<T> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Refer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct ReferMany<T>(Vec<usize>, PhantomData<fn() -> T>);
impl<T> ReferMany<T> {
    pub fn to(inds: Vec<usize>) -> Self {
        Self(inds, PhantomData)
    }
}
impl<T> Deref for ReferMany<T> {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for ReferMany<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn store<T>(store: &mut Store<T>, val: T) -> Refer<T> {
    Refer::to(store.insert(val))
}

pub fn store_primary<T>(store: &mut Store<T>, val: T) -> Refer<T> {
    Refer::to(store.insert_primary(val))
}

pub fn store_many<T>(store: &mut Store<T>, mut vals: Vec<T>) -> ReferMany<T> {
    let mut inds = Vec::with_capacity(vals.len());
    for val in vals.drain(..) {
        inds.push(store.insert(val));
    }
    ReferMany::to(inds)
}

pub fn blue_noise_image(w: u32, h: u32) -> Vec<u8> {
    let mut noise = BlueNoise::<Pcg64Mcg>::new(w as f32, h as f32, 5.0);
    let noise_black = noise.with_samples(w * (h / 3)).with_seed(10);

    let mut noise2 = BlueNoise::<Pcg64Mcg>::new(w as f32, h as f32, 5.0);
    let noise_gray = noise2.with_samples(w * (h / 3)).with_seed(20);

    let mut img: Vec<u8> = vec![0; (w * h) as usize];

    for p in noise_black {
        img[(p.y as u32 * w + p.x as u32) as usize] = 255;
    }
    let mut c = 0;
    for p in noise_gray {
        if p.y as u32 * w + p.x as u32 == 255 {
            break;
        }
        c += 1;
        img[(p.y as u32 * w + p.x as u32) as usize] = 127;
    }
    dbg!(c);

    img
}
