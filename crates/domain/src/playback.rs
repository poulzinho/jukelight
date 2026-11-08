use std::time::Duration;

use crate::{AudioQuality, Delivery, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Buffering,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Track {
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
}

/// Where a piece of artwork comes from. Opaque to the domain: only the
/// adapter that created it, and the artwork adapter, look inside.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtworkRef(String);

impl ArtworkRef {
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn source(&self) -> &str {
        &self.0
    }
}

/// Elapsed time within the current track, and when it was observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub elapsed: Duration,
    pub duration: Option<Duration>,
    pub observed_at: Timestamp,
}

impl Position {
    /// Elapsed time expected at `at`: it advances only while playing and
    /// never passes the end of the track.
    pub fn predict(&self, at: Timestamp, state: PlaybackState) -> Duration {
        let advanced = if state == PlaybackState::Playing {
            self.elapsed + at.since(self.observed_at)
        } else {
            self.elapsed
        };
        match self.duration {
            Some(duration) => advanced.min(duration.max(self.elapsed)),
            None => advanced,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueueEntry {
    pub title: String,
    pub artists: Vec<String>,
    pub artwork: Option<ArtworkRef>,
}

/// Upcoming entries, or the fact that this session has no queue to show.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueueView {
    Available(Vec<QueueEntry>),
    NotApplicable,
}

/// A brand, such as a music service or a player's maker: a short lowercase
/// identifier (`qobuz`, `apple-music`, `bluesound`). It also names the
/// brand's logo file, so it can't contain anything that leaves a folder.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Brand(String);

impl Brand {
    /// The brand `id`, if it is lowercase letters, digits and inner hyphens.
    pub fn new(id: &str) -> Option<Self> {
        let valid = !id.is_empty()
            && !id.starts_with('-')
            && !id.ends_with('-')
            && id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        valid.then(|| Self(id.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A music service or another source, with its brand when it is known.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Service {
    pub brand: Option<Brand>,
    /// The name to show, as the source names it.
    pub name: String,
}

impl Service {
    pub fn new(brand: Option<Brand>, name: impl Into<String>) -> Self {
        Self {
            brand,
            name: name.into(),
        }
    }
}

/// What carries the music to the player, such as Music Assistant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    pub service: Service,
    /// Whether it can be reached.
    pub available: bool,
}

/// Where the music comes from: a service, and the route it takes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Source {
    pub service: Service,
    pub via: Option<Route>,
}

/// Where the device's audio comes from, as the device reports it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Origin {
    /// The device plays a stream from a URL.
    Stream {
        url: String,
    },
    /// The device plays from a service; `cast` when an app elsewhere drives it.
    Service {
        id: String,
        name: String,
        cast: bool,
        /// The service's brand, when it is known.
        brand: Option<Brand>,
    },
    /// The device plays one of its audio inputs.
    Input(AudioInput),
    None,
}

/// The kind of an audio input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKind {
    Optical,
    Analog,
    HdmiArc,
    Other,
}

/// An audio input the player plays: nobody knows what comes in on it, only
/// which input it is and, once a signal arrived, since when it plays. The
/// player notices when a signal starts, not when it stops.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AudioInput {
    pub kind: InputKind,
    /// The name the user gave the input in the player's app.
    pub name: String,
    /// When the signal started; `None` while waiting for a signal.
    pub started_at: Option<Timestamp>,
}

/// A colour as red, green and blue from 0 to 255.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// The album's colours, when a source knows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Palette {
    pub background: Rgb,
    pub accent: Rgb,
}

/// What is playing, from whichever source knows it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Content {
    pub track: Track,
    pub artwork: Option<ArtworkRef>,
    pub position: Option<Position>,
    /// The quality as the source delivers it.
    pub quality: Option<AudioQuality>,
    /// What the route sends on to the player, when a provider knows it.
    pub delivery: Option<Delivery>,
    pub queue: QueueView,
    /// The music service behind the session, e.g. Qobuz, when a provider
    /// knows it.
    pub service: Option<Service>,
    pub palette: Option<Palette>,
}

/// What the device reports about itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceSnapshot {
    pub name: String,
    /// Stable identity of the device, e.g. its MAC address without separators.
    pub player_key: String,
    pub online: bool,
    pub state: PlaybackState,
    pub origin: Origin,
    /// The content the device itself knows, if any.
    pub content: Option<Content>,
    /// The player's maker, when it is a known brand.
    pub maker: Option<Brand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Session {
    pub label: String,
    pub available: bool,
}

/// A player's current activity: the single picture the kiosk shows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerSnapshot {
    pub player_name: String,
    pub online: bool,
    pub state: PlaybackState,
    pub session: Session,
    pub content: Option<Content>,
    /// The audio input, while one plays.
    pub input: Option<AudioInput>,
    /// Where the music comes from, when it is known.
    pub source: Option<Source>,
    /// The player's maker, when it is a known brand.
    pub maker: Option<Brand>,
}

impl PlayerSnapshot {
    /// The snapshot before anything is known about the player.
    pub fn nothing_playing(player_name: impl Into<String>) -> Self {
        Self {
            player_name: player_name.into(),
            online: false,
            state: PlaybackState::Stopped,
            session: Session::default(),
            content: None,
            input: None,
            source: None,
            maker: None,
        }
    }
}
