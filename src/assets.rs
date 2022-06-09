//! Asset management.

use bftd_lib::Metadata;

use anyhow::Error;

use crate::battle::fsm::{Frame, Fsm, Key, State};
use crate::render::Texture;
use crate::Context;

use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::sync::{Arc, Weak};

/// An asset's type.
pub type Asset<T> = Arc<T>;

/// An asset bundle.
pub struct Bundle {
    metadata: Metadata,
    cache: HashMap<String, Weak<dyn Any + Send + Sync>>,
    path: PathBuf,
}

impl Bundle {
    /// Creates a new [`Bundle`] from a directory.
    pub fn new(path: impl Into<PathBuf>) -> Result<Bundle, Error> {
        let path = path.into();

        // load the metadata
        let metadata = File::open(path.join("bundle.ron"))?;
        let metadata = ron::de::from_reader(metadata)?;

        Ok(Bundle {
            metadata,
            cache: HashMap::new(),
            path,
        })
    }

    /// Loads a file from the bundle.
    ///
    /// This loads from the bundle's cache if the resource is cached.
    pub fn load<T>(&mut self, cx: &mut Context, path: &str) -> Result<Asset<T>, Error>
    where
        T: Loadable + Send + Sync + 'static,
    {
        if let Some(cached) = self.cache.get(path).and_then(|s| s.upgrade()) {
            if let Ok(cached) = cached.downcast() {
                return Ok(cached);
            }
        }

        debug!(
            "loading file \"{}\" from bundle {}...",
            path, self.metadata.name
        );

        // clip leading slash, if there is any
        let path = path.trim_start_matches('/');
        let data = T::load(cx, File::open(self.path.join(path))?).map(Arc::new)?;

        {
            let data: Arc<dyn Any + Send + Sync + 'static> = data.clone();
            self.cache.insert(path.to_owned(), Arc::downgrade(&data));
        }

        Ok(data)
    }

    /// Loads a character from a bundle.
    pub fn load_character(&mut self, cx: &mut Context, path: &str) -> Result<Fsm, Error> {
        let character = self.load::<bftd_lib::Character>(cx, path)?;

        let mut states = Vec::new();
        for state in character.states.iter() {
            // load script if necessary
            let script = match &state.script {
                Some(path) => {
                    let script = self.load::<String>(cx, path)?;

                    // compile script
                    let ast = cx.script.compile(script.as_str())?;

                    Some(ast)
                }
                None => None,
            };

            let mut frames = Vec::new();
            for frame in state.frames.iter() {
                // load sprite if necessary
                let sprite = match &frame.sprite {
                    Some(sprite) => {
                        use std::ops::Deref as _;
                        let texture = self.load::<Texture>(cx, &sprite.texture)?;

                        // FIXME: possibly bad if we avoid asset handling Arcs
                        Some(texture.deref().clone().into())
                    }
                    None => None,
                };

                frames.push(Frame { sprite });
            }

            states.push(State {
                name: Key::from(state.name.as_str()),
                frames,
                script,
            });
        }

        Ok(Fsm::new(states))
    }

    /// The metadata of the bundle.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

/// An asset that can be loaded from a [`Bundle`].
pub trait Loadable: Sized {
    /// Loads an asset from a stream.
    fn load<T>(cx: &mut Context, stream: T) -> Result<Self, Error>
    where
        T: Read + Seek;
}

impl Loadable for String {
    fn load<T>(_cx: &mut Context, mut stream: T) -> Result<Self, Error>
    where
        T: Read,
    {
        let mut buf = String::new();

        stream.read_to_string(&mut buf)?;

        Ok(buf)
    }
}

impl Loadable for Texture {
    fn load<R>(cx: &mut Context, stream: R) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        cx.render.load_texture(stream)
    }
}

macro_rules! impl_ron {
    ($T:ty) => {
        impl Loadable for $T {
            fn load<R>(_cx: &mut Context, stream: R) -> Result<Self, Error>
            where
                R: Read,
            {
                ron::de::from_reader(stream).map_err(From::from)
            }
        }
    };
}

impl_ron!(bftd_lib::Character);
