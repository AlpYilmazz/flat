use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy::{asset::HandleId, ecs::system::SystemParam, prelude::*};

// TODO: Maybe change to Vec
//       Vec -> more cache friendly, worse removal
//       HashMap -> not cache friendly, better removal
pub struct Store<T> {
    ind: usize,
    inner: HashMap<usize, T>,
    // primary: Option<T>,
    // inner: Vec<T>,
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

    pub fn contains_key(&self, key: usize) -> bool {
        self.inner.contains_key(&key)
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

    pub fn contains_primary(&self) -> bool {
        self.inner.contains_key(&Self::PRIMARY_ID)
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

pub type AssetBound<T> = HashMap<HandleId, T>;
pub type EntityBound<T> = HashMap<Entity, T>;

#[derive(Resource)]
pub struct NewTypePhantom<V, T>(pub V, PhantomData<T>);
impl<V: Default, T> Default for NewTypePhantom<V, T> {
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}
impl<V, T> Deref for NewTypePhantom<V, T> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<V, T> DerefMut for NewTypePhantom<V, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// #[derive(Resource)]
// pub struct AssetBound<T>(pub HashMap<HandleId, T>);
// impl<T> Default for AssetBound<T> {
//     fn default() -> Self {
//         Self(Default::default())
//     }
// }
// impl<T> Deref for AssetBound<T> {
//     type Target = HashMap<HandleId, T>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl<T> DerefMut for AssetBound<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// pub struct EntityBound<T>(pub HashMap<Entity, T>);
// impl<T> Default for EntityBound<T> {
//     fn default() -> Self {
//         Self(Default::default())
//     }
// }
// impl<T> Deref for EntityBound<T> {
//     type Target = HashMap<Entity, T>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl<T> DerefMut for EntityBound<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

#[derive(Component)]
pub struct Refer<T>(usize, PhantomData<fn() -> T>);
impl<T> Refer<T> {
    pub fn to(ind: usize) -> Self {
        Self(ind, PhantomData)
    }
}
impl<T> Clone for Refer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}
impl<T> Copy for Refer<T> {}
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
impl<T> Clone for ReferMany<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
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

#[derive(Resource)]
pub struct PrimaryEntity<T> {
    pub entity: Entity,
    _marker: PhantomData<fn() -> T>,
}

impl<T> PrimaryEntity<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}

#[derive(SystemParam)]
pub struct Primary<'w, 's, T: 'static> {
    pub inner: Res<'w, PrimaryEntity<T>>,
    #[system_param(ignore)]
    _marker: PhantomData<&'s usize>,
}

impl<'w, 's, T> Primary<'w, 's, T> {
    pub fn get(&self) -> Entity {
        self.entity
    }
}

impl<'w, 's, T> Deref for Primary<'w, 's, T> {
    type Target = Res<'w, PrimaryEntity<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'w, 's, T> AsRef<Entity> for Primary<'w, 's, T> {
    #[inline]
    fn as_ref(&self) -> &Entity {
        &self.entity
    }
}

#[derive(SystemParam)]
pub struct PrimaryMut<'w, 's, T: 'static> {
    pub inner: ResMut<'w, PrimaryEntity<T>>,
    #[system_param(ignore)]
    _marker: PhantomData<&'s usize>,
}

impl<'w, 's, T> PrimaryMut<'w, 's, T> {
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

impl<'w, 's, T> Deref for PrimaryMut<'w, 's, T> {
    type Target = ResMut<'w, PrimaryEntity<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'w, 's, T> DerefMut for PrimaryMut<'w, 's, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'w, 's, T> AsRef<Entity> for PrimaryMut<'w, 's, T> {
    #[inline]
    fn as_ref(&self) -> &Entity {
        &self.entity
    }
}
impl<'w, 's, T> AsMut<Entity> for PrimaryMut<'w, 's, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

pub trait EngineDefault {
    fn engine_default() -> Self;
}

impl EngineDefault for wgpu::TextureFormat {
    fn engine_default() -> Self {
        wgpu::TextureFormat::Bgra8UnormSrgb
    }
}

pub trait Sink: Sized {
    fn sink(self) {}
}
impl<T: Sized> Sink for T {}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Label(pub String);
impl Deref for Label {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Label {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait Labeled {
    fn get_label(&self) -> &Option<Label>;
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location(pub u32);
impl Deref for Location {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Location {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait LocationBound {
    fn get_location(&self) -> Location;
}
