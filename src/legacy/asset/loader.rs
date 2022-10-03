use std::{path::{PathBuf, Path}, env, future::Future, pin::Pin, fs::File, io::Read};

use super::{lifecycle::AssetLifecycle, Asset};


type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct FileAssetIo {
    root: PathBuf,
}

impl FileAssetIo {
    pub fn new() -> Self {
        Self {
            root: Self::get_root_path(),
        }
    }

    pub fn get_root_path() -> PathBuf {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest_dir)
        } else {
            env::current_exe()
                .map(|path| {
                    path.parent()
                        .map(|exe_parent_path| exe_parent_path.to_owned())
                        .unwrap()
                })
                .unwrap()
        }
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root
    }

    pub fn load_file<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            let full_path = self.root.join(path);
            match File::open(&full_path) {
                Ok(mut file) => {
                    file.read_to_end(&mut bytes).unwrap();//?;
                }
                Err(_e) => {
                    // return if e.kind() == std::io::ErrorKind::NotFound {
                    //     Err(AssetIoError::NotFound(full_path))
                    // } else {
                    //     Err(e.into())
                    // }
                    panic!("Err file io");
                }
            }
            bytes
            // Ok(bytes)
        })
    }
}

pub trait AssetLoader: Send + Sync + 'static {
    type LoadedAsset: Asset;

    fn load(&self, bytes: &[u8]) -> Option<Self::LoadedAsset>;
}

pub struct AssetHandler<T: AssetLoader> {
    pub(super) loader: T,
    pub(super) lifecycle: AssetLifecycle<T::LoadedAsset>,
}

impl<T: AssetLoader> AssetHandler<T> {
    pub fn new(loader: T) -> Self {
        Self {
            loader,
            lifecycle: AssetLifecycle::new(),
        }
    }
}


pub struct Bytes(Vec<u8>);
pub struct BytesLoader {

}

impl BytesLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl AssetLoader for BytesLoader {
    type LoadedAsset = Bytes;

    fn load(&self, bytes: &[u8]) -> Option<Self::LoadedAsset> {
        Some(Bytes(bytes.to_owned()))
    }
}