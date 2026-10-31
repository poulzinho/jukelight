use std::time::Duration;

use crate::{AudioQuality, Timestamp};

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
/// which input it is and since when it is selected.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AudioInput {
    pub kind: InputKind,
    /// The name the user gave the input in the player's app.
    pub name: String,
    pub selected_at: Timestamp,
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
    pub quality: Option<AudioQuality>,
    pub queue: QueueView,
    /// The music service behind the session, e.g. "Qobuz", when a provider
    /// knows it.
    pub service: Option<String>,
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
        }
    }
}
