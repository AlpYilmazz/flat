use std::{path::Path, sync::Arc, collections::HashMap, marker::PhantomData, hash::{Hash, Hasher}};

use ahash::AHasher;
use crossbeam_channel::TryRecvError;

use self::{task::TaskPool, loader::{FileAssetIo, AssetLoader, AssetHandler, BytesLoader}, lifecycle::AssetLifecycle};


pub mod task;
pub mod loader;
pub mod lifecycle;


pub trait Asset: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Asset for T {}

pub enum AssetKind {
    Bytes,
    Image,
    Audio,
}

pub struct AssetHandlers {
    for_bytes: AssetHandler<BytesLoader>,
}

impl AssetHandlers {
    pub fn new() -> Self {
        Self {
            for_bytes: AssetHandler::new(BytesLoader::new()),
        }
    }
}

pub struct AssetServerInner {
    task_pool: TaskPool,
    asset_io: FileAssetIo,
    handlers: AssetHandlers,
}

#[derive(Clone)]
pub struct AssetServer {
    server: Arc<AssetServerInner>,
}

impl AssetServer {
    pub fn new() -> Self {
        Self {
            server: Arc::new(AssetServerInner {
                task_pool: TaskPool::default(),
                asset_io: FileAssetIo::new(),
                handlers: AssetHandlers::new(),
            })
        }
    }

    pub async fn load_async(&self, path: String, kind: AssetKind) {
        let bytes = self.server.asset_io.load_file(Path::new(&path)).await;
        match kind {
            AssetKind::Bytes => {
                let handler = &self.server.handlers.for_bytes;
                let asset = handler.loader.load(&bytes).unwrap();
                handler.lifecycle.create(asset);
            },
            AssetKind::Image => todo!(),
            AssetKind::Audio => todo!(),
        }
    }

    pub fn load(&self, path: &str, kind: AssetKind) {
        let server = self.clone();
        let owned_path = path.to_owned();
        self.server
            .task_pool
            .spawn(async move {
                server.load_async(owned_path, kind).await;
            })
            .detach();
    }

    pub fn load_bytes(&self, path: &str) {
        self.load(path, AssetKind::Bytes)
    }

    // pub fn get_bytes(&self) -> Option<Vec<u8>> {
    //     let receiver = &self.server.asset_lifecycle.receiver;
    //     match receiver.try_recv() {
    //         Ok(bytes) => Some(bytes),
    //         Err(TryRecvError::Empty) => None,
    //         Err(TryRecvError::Disconnected) => {
    //             panic!("Async channel disconnected");
    //         },
    //     }
    // }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct HandleId {
    // from path and label
    path_id: u64,
    label_id: u64,
}

impl HandleId {
    pub fn from(path: &str, label: &str) -> Self {
        Self {
            path_id: hashed(path),
            label_id: hashed(label),
        }
    }
}

fn hashed(s: &str) -> u64 {
    let mut hasher = get_hasher();
    s.hash(&mut hasher);
    hasher.finish()
}

/// this hasher provides consistent results across runs
fn get_hasher() -> AHasher {
    AHasher::new_with_keys(42, 23)
}

impl<T: Asset> From<Handle<T>> for HandleId {
    fn from(val: Handle<T>) -> Self {
        val.id
    }
}

pub struct Handle<T: Asset> {
    id: HandleId,
    // type: HandleType, // for weak and strong Handles and ref counting
    _marker: PhantomData<fn() -> T>,
}

pub struct Assets<T: Asset> {
    store: HashMap<HandleId, T>,
    // assets are async loaded, loads and such trigger events, for bevy
    // events: Events<AssetEvents<T>>,
}

impl<T: Asset> Assets<T> {
    pub fn new() -> Self {
         Self {
            store: Default::default(),
         }
    }

    pub fn insert(&mut self, handle: Handle<T>, asset: T) {
        self.store.insert(handle.into(), asset);
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.store.get(&handle.into())
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.store.get_mut(&handle.into())
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        self.store.remove(&handle.into());
    }
}

