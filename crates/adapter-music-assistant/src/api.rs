//! The parts of Music Assistant's API responses the provider reads. Every
//! field is optional and unknown fields are ignored, so other Music
//! Assistant versions still parse.

use serde::Deserialize;

/// `players/get`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Player {
    pub current_media: Option<CurrentMedia>,
}

/// What the player reports as playing; the fallback for the queue.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct CurrentMedia {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub image_url: Option<String>,
    pub duration: Option<f64>,
    pub palette: Option<Palette>,
}

/// Colours Music Assistant derives from the cover (Sendspin `color@v1`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Palette {
    pub background_dark: Option<[u8; 3]>,
    pub on_dark: Option<[u8; 3]>,
}

/// `player_queues/get`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Queue {
    pub current_index: Option<u32>,
    pub current_item: Option<QueueItem>,
    /// Number of items in the queue.
    pub items: u32,
    /// Seconds into the current track.
    pub elapsed_time: Option<f64>,
    /// Unix time in seconds when `elapsed_time` was measured.
    pub elapsed_time_last_updated: Option<f64>,
}

/// An entry of the queue (`current_item`, `player_queues/items`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct QueueItem {
    pub queue_item_id: String,
    pub name: String,
    pub duration: Option<f64>,
    pub media_item: Option<MediaItem>,
    pub image: Option<Image>,
    pub streamdetails: Option<StreamDetails>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct MediaItem {
    pub name: String,
    pub duration: Option<f64>,
    pub artists: Vec<Named>,
    pub album: Option<Album>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Named {
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Album {
    pub name: String,
    pub image: Option<Image>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Metadata {
    pub images: Option<Vec<Image>>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Image {
    pub path: String,
    pub remotely_accessible: bool,
    pub proxy_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct StreamDetails {
    /// Instance id of the music provider, e.g. `qobuz--abc123`.
    pub provider: Option<String>,
    pub audio_format: Option<AudioFormat>,
    pub audio_processing: Option<AudioProcessing>,
    pub volume_normalization_mode: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct AudioFormat {
    pub content_type: String,
    pub sample_rate: u32,
    pub bit_depth: u32,
    /// kbit/s; 0 when unknown.
    pub bit_rate: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct AudioProcessing {
    pub outputs: Vec<Output>,
}

/// What Music Assistant sends to one or more players.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Output {
    pub player_ids: Vec<String>,
    pub output_format: Option<AudioFormat>,
}

/// An entry of `providers`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct ProviderInfo {
    pub instance_id: String,
    pub name: String,
}
