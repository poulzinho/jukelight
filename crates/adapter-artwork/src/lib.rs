//! Artwork proxy: issues opaque ids for artwork sources and serves the
//! images itself, so the browser never learns where artwork comes from.

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::FutureExt;
use futures::future::BoxFuture;
use jukelight_application::{ArtworkError, ArtworkId, ArtworkSize, ArtworkSource, Image};
use jukelight_domain::ArtworkRef;
use sha2::{Digest, Sha256};

/// Images kept in memory by default.
const DEFAULT_CAPACITY: usize = 64;
/// The longer side of a backdrop image, in pixels: stretched over the
/// screen, it blurs by itself, with no work for the GPU.
const BACKDROP_PIXELS: u32 = 32;
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
    cdn: CdnHosts,
}

/// The image CDNs whose URLs name a larger version of a cover. Their URL
/// conventions are undocumented, so a larger version is only ever tried,
/// with the original as the fallback.
struct CdnHosts {
    qobuz: Cow<'static, str>,
    spotify: Cow<'static, str>,
}

impl CdnHosts {
    const REAL: Self = Self {
        qobuz: Cow::Borrowed("static.qobuz.com"),
        spotify: Cow::Borrowed("i.scdn.co"),
    };

    /// The URL of a larger version of the image at `url`, if its CDN is
    /// known: Qobuz `…_600.jpg` (any size) becomes `…_max.jpg` (up to
    /// 3000 px), Spotify's 640 px size code `ab67616d0000b273` becomes
    /// `ab67616d000082c1` (about 1400 to 1650 px).
    fn large_version(&self, url: &str) -> Option<String> {
        let mut url = reqwest::Url::parse(url).ok()?;
        let host = url.host_str()?;
        let authority = match url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_owned(),
        };
        let path = url.path();
        let larger = (authority == self.qobuz)
            .then(|| qobuz_large(path))
            .flatten()
            .or_else(|| {
                (authority == self.spotify)
                    .then(|| spotify_large(path))
                    .flatten()
            })?;
        url.set_path(&larger);
        Some(url.into())
    }
}

/// `…_<size>.jpg` as `…_max.jpg`.
fn qobuz_large(path: &str) -> Option<String> {
    let (stem, size) = path.strip_suffix(".jpg")?.rsplit_once('_')?;
    (!size.is_empty() && size.bytes().all(|b| b.is_ascii_digit()))
        .then(|| format!("{stem}_max.jpg"))
}

/// Spotify's 640 px cover with the size code of the largest one.
fn spotify_large(path: &str) -> Option<String> {
    let id = path.strip_prefix("/image/ab67616d0000b273")?;
    Some(format!("/image/ab67616d000082c1{id}"))
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
                cdn: CdnHosts::REAL,
            }),
        }
    }

    /// Treats `qobuz` and `spotify` (`host:port`) as the image CDNs, so tests
    /// can stand in for them. Only before the proxy is shared.
    #[doc(hidden)]
    #[must_use]
    pub fn with_cdn_hosts(mut self, qobuz: &str, spotify: &str) -> Self {
        if let Some(inner) = Arc::get_mut(&mut self.inner) {
            inner.cdn = CdnHosts {
                qobuz: Cow::Owned(qobuz.to_owned()),
                spotify: Cow::Owned(spotify.to_owned()),
            };
        }
        self
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

    fn fetch(
        &self,
        id: &ArtworkId,
        size: ArtworkSize,
    ) -> BoxFuture<'static, Result<Image, ArtworkError>> {
        let inner = Arc::clone(&self.inner);
        let id = id.as_str().to_owned();
        async move {
            let source = lock(&inner.issued).get(&id).ok_or(ArtworkError::Unknown)?;
            // Each size of an image is cached on its own.
            let key = format!("{id}:{}", size_key(size));
            if let Some(image) = lock(&inner.cache).get(&key) {
                return Ok(image);
            }
            let image = inner.load_size(&source, size).await?;
            lock(&inner.cache).insert(key, image.clone());
            Ok(image)
        }
        .boxed()
    }
}

impl Inner {
    /// The image in the requested size.
    async fn load_size(&self, source: &str, size: ArtworkSize) -> Result<Image, ArtworkError> {
        match size {
            ArtworkSize::Original => Ok(self.load(source).await?.0),
            ArtworkSize::Backdrop => backdrop(&self.load(source).await?.0),
            ArtworkSize::Large => {
                // A CDN URL can be upgraded before fetching anything.
                if let Some(larger) = self.cdn.large_version(source)
                    && let Ok((image, _)) = self.load(&larger).await
                    && image.content_type.starts_with("image/")
                {
                    return Ok(image);
                }
                let (original, location) = self.load(source).await?;
                // The rules apply to where the source led, for example from a
                // player's /Artwork URL to a CDN.
                if let Some(larger) = location.and_then(|url| self.cdn.large_version(&url))
                    && let Ok((image, _)) = self.load(&larger).await
                    && image.content_type.starts_with("image/")
                {
                    return Ok(image);
                }
                Ok(original)
            }
        }
    }

    /// The image at `source`, and the URL it was finally served from.
    async fn load(&self, source: &str) -> Result<(Image, Option<String>), ArtworkError> {
        if let Some(data) = source.strip_prefix("data:") {
            return Ok((decode_data_url(data)?, None));
        }
        // reqwest follows redirects, e.g. from a player to an image CDN.
        let response = self.client.get(source).send().await.map_err(source_error)?;
        let location = response.url().to_string();
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
        Ok((
            Image {
                content_type,
                bytes,
            },
            Some(location),
        ))
    }
}

/// The image scaled down to at most `BACKDROP_PIXELS` on its longer side,
/// as PNG. Fails like an unreachable source when it isn't a raster image.
fn backdrop(image: &Image) -> Result<Image, ArtworkError> {
    let small = image::load_from_memory(&image.bytes)
        .map_err(source_error)?
        .thumbnail(BACKDROP_PIXELS, BACKDROP_PIXELS);
    let mut png = Vec::new();
    small
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(source_error)?;
    Ok(Image {
        content_type: "image/png".into(),
        bytes: png,
    })
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

fn size_key(size: ArtworkSize) -> &'static str {
    match size {
        ArtworkSize::Original => "original",
        ArtworkSize::Large => "large",
        ArtworkSize::Backdrop => "backdrop",
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const HOSTS: CdnHosts = CdnHosts::REAL;

    #[test]
    fn qobuz_cover() {
        assert_eq!(
            HOSTS
                .large_version("https://static.qobuz.com/images/covers/ab/cd/abcdef_600.jpg")
                .as_deref(),
            Some("https://static.qobuz.com/images/covers/ab/cd/abcdef_max.jpg")
        );
    }

    #[test]
    fn spotify_cover() {
        assert_eq!(
            HOSTS
                .large_version("https://i.scdn.co/image/ab67616d0000b2731234567890abcdef12345678")
                .as_deref(),
            Some("https://i.scdn.co/image/ab67616d000082c11234567890abcdef12345678")
        );
    }

    #[test]
    fn unknown_sources_and_shapes() {
        for url in [
            "https://example.com/images/covers/ab/cd/abcdef_600.jpg",
            "https://static.qobuz.com/images/covers/ab/cd/abcdef_max.jpg",
            "https://static.qobuz.com/images/covers/ab/cd/abcdef.jpg",
            "https://i.scdn.co/image/ab6761610000e5eb1234567890abcdef12345678",
            "http://192.168.1.40:11000/Artwork?songid=1",
        ] {
            assert_eq!(HOSTS.large_version(url), None, "{url}");
        }
    }
}
