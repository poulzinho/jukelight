//! The parts of Music Assistant's API responses the provider reads. Every
//! field is optional and unknown fields are ignored, so other Music
//! Assistant versions still parse.

use serde::Deserialize;

/// `players/get`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Player {
    pub current_media: Option<CurrentMedia>,
    /// What the player plays from: a queue id (a player id) when Music
    /// Assistant drives it, otherwise e.g. `AirPlay`.
    pub active_source: Option<String>,
    /// The group leader's player id, when this player is a group member.
    pub synced_to: Option<String>,
    /// The group's player ids, this one included, when this player leads.
    pub group_members: Vec<String>,
}

/// An entry of `players/all`: what the provider needs about another player
/// of the group.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct PlayerInfo {
    pub player_id: String,
    pub provider: String,
    pub name: String,
    pub display_name: Option<String>,
    /// `playing`, `paused` or `idle`.
    pub playback_state: Option<String>,
    pub synced_to: Option<String>,
    pub group_members: Vec<String>,
    pub device_info: Option<DeviceInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct DeviceInfo {
    pub manufacturer: Option<String>,
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
    /// `playing`, `paused` or `idle`; a paused group's queue is `idle`.
    pub state: Option<String>,
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
    /// The normalisation gain in dB (older releases; see `Normalization`).
    pub volume_normalization_gain_correct: Option<f64>,
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
    pub queue_processing: Option<QueueProcessing>,
    pub outputs: Vec<Output>,
}

/// What Music Assistant does to the queue's audio before it splits it per
/// player.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct QueueProcessing {
    pub normalization: Option<Normalization>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Normalization {
    pub applied_gain_db: Option<f64>,
}

/// A player's DSP (equaliser, filters); `state` is `enabled` when it
/// changes the audio.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Dsp {
    pub state: String,
}

/// What Music Assistant sends to one or more players.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Output {
    pub player_ids: Vec<String>,
    pub output_format: Option<AudioFormat>,
    pub dsp: Option<Dsp>,
}

/// An entry of `providers`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct ProviderInfo {
    pub instance_id: String,
    pub name: String,
    /// The provider's type, e.g. `qobuz` or `opensubsonic`, from which the
    /// brand is derived (not always the brand's identifier itself).
    pub domain: String,
}
