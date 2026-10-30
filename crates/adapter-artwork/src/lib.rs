//! Artwork proxy: issues opaque ids for artwork sources and serves the
//! images itself, so the browser never learns where artwork comes from.

use std::collections::{HashMap, VecDeque};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::FutureExt;
use futures::future::BoxFuture;
use jukelight_application::{ArtworkError, ArtworkId, ArtworkSource, Image};
use jukelight_domain::ArtworkRef;
use sha2::{Digest, Sha256};

/// Images kept in memory by default.
const DEFAULT_CAPACITY: usize = 64;
/// Issued ids remembered; the oldest are forgotten first.
const MAX_ISSUED: usize = 4096;
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub struct ArtworkProxy {
    inner: Arc<Inner>,
}

struct Inner {
    issued: Mutex<Bounded<String>>,
    cache: Mutex<Bounded<Image>>,
    client: reqwest::Client,
}

impl ArtworkProxy {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// A proxy that keeps at most `capacity` images in memory.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                issued: Mutex::new(Bounded::new(MAX_ISSUED)),
                cache: Mutex::new(Bounded::new(capacity)),
                client: reqwest::Client::builder()
                    .timeout(FETCH_TIMEOUT)
                    .build()
                    .unwrap_or_default(),
            }),
        }
    }
}

impl Default for ArtworkProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtworkSource for ArtworkProxy {
    fn issue(&self, artwork: &ArtworkRef) -> ArtworkId {
        let digest = Sha256::digest(artwork.source().as_bytes());
        let id = digest
            .iter()
            .take(8)
            .fold(String::with_capacity(16), |mut id, byte| {
                let _ = write!(id, "{byte:02x}");
                id
            });
        lock(&self.inner.issued).insert(id.clone(), artwork.source().to_owned());
        ArtworkId::new(id)
    }

    fn fetch(&self, id: &ArtworkId) -> BoxFuture<'static, Result<Image, ArtworkError>> {
        let inner = Arc::clone(&self.inner);
        let id = id.as_str().to_owned();
        async move {
            let source = lock(&inner.issued).get(&id).ok_or(ArtworkError::Unknown)?;
            if let Some(image) = lock(&inner.cache).get(&id) {
                return Ok(image);
            }
            let image = inner.load(&source).await?;
            lock(&inner.cache).insert(id, image.clone());
            Ok(image)
        }
        .boxed()
    }
}

impl Inner {
    async fn load(&self, source: &str) -> Result<Image, ArtworkError> {
        if let Some(data) = source.strip_prefix("data:") {
            return decode_data_url(data);
        }
        // reqwest follows redirects, e.g. from a player to an image CDN.
        let response = self.client.get(source).send().await.map_err(source_error)?;
        if !response.status().is_success() {
            return Err(ArtworkError::Source(format!(
                "source answered {}",
                response.status()
            )));
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();
        let bytes = response.bytes().await.map_err(source_error)?.to_vec();
        Ok(Image {
            content_type,
            bytes,
        })
    }
}

/// `data:<type>;base64,<data>`.
fn decode_data_url(data: &str) -> Result<Image, ArtworkError> {
    let (meta, payload) = data
        .split_once(',')
        .ok_or_else(|| ArtworkError::Source("malformed data URL".into()))?;
    let content_type = meta
        .strip_suffix(";base64")
        .ok_or_else(|| ArtworkError::Source("only base64 data URLs are supported".into()))?;
    let bytes = BASE64.decode(payload).map_err(source_error)?;
    Ok(Image {
        content_type: content_type.to_owned(),
        bytes,
    })
}

fn source_error(error: impl std::fmt::Display) -> ArtworkError {
    ArtworkError::Source(error.to_string())
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A map that keeps at most `capacity` entries, forgetting the least
/// recently used one first.
struct Bounded<V> {
    capacity: usize,
    values: HashMap<String, V>,
    order: VecDeque<String>,
}

impl<V: Clone> Bounded<V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            values: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get(&mut self, key: &str) -> Option<V> {
        let value = self.values.get(key)?.clone();
        self.touch(key);
        Some(value)
    }

    fn insert(&mut self, key: String, value: V) {
        if self.values.insert(key.clone(), value).is_some() {
            self.touch(&key);
            return;
        }
        self.order.push_back(key);
        while self.order.len() > self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.values.remove(&oldest);
            }
        }
    }

    fn touch(&mut self, key: &str) {
        if let Some(position) = self.order.iter().position(|k| k == key)
            && let Some(key) = self.order.remove(position)
        {
            self.order.push_back(key);
        }
    }
}
