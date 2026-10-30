use std::time::{SystemTime, UNIX_EPOCH};

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use jukelight_domain::{ArtworkRef, Content, DeviceSnapshot, Timestamp};

/// Follows one device and reports what it is doing.
pub trait DeviceMonitor: Send + Sync + 'static {
    /// Device snapshots, starting now. Ends only when the monitor stops.
    fn snapshots(&self) -> BoxStream<'static, DeviceSnapshot>;
}

/// Something that can drive a device (a session) and knows its content.
pub trait SessionProvider: Send + Sync + 'static {
    /// Shown as the session label, e.g. "Music Assistant".
    fn label(&self) -> &str;

    /// Whether this provider is driving the device. Decided from the
    /// provider's configuration and the device snapshot alone, without
    /// contacting the provider.
    fn claims(&self, device: &DeviceSnapshot) -> bool;

    /// The provider's current content, or `None` while it is unreachable.
    fn content(&self) -> Option<Content>;

    /// The current device report, given before every rebuild of the player
    /// snapshot, so the provider can start or stop following the player.
    fn follow(&self, device: &DeviceSnapshot);

    /// Ticks whenever `content()` has changed, so the snapshot can be rebuilt
    /// even when the device reports nothing new.
    fn updates(&self) -> BoxStream<'static, ()>;
}

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> Timestamp;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
            });
        Timestamp::from_millis(millis)
    }
}

/// Opaque id under which the backend serves a piece of artwork.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtworkId(String);

impl ArtworkId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtworkError {
    /// The id was never issued.
    Unknown,
    /// The artwork's source failed.
    Source(String),
}

/// Issues opaque artwork ids and serves the images behind them.
pub trait ArtworkSource: Send + Sync + 'static {
    fn issue(&self, artwork: &ArtworkRef) -> ArtworkId;
    fn fetch(&self, id: &ArtworkId) -> BoxFuture<'static, Result<Image, ArtworkError>>;
}
