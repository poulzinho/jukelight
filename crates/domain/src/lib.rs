//! Jukelight's playback model, in its own language.
//!
//! What a player is doing (a snapshot), how its position moves, how good the
//! audio is, which events follow from one snapshot to the next, and when the
//! display should be on. No IO, no vendor terms: adapters translate into
//! these types.

mod changes;
mod display;
mod playback;
mod quality;
mod time;

pub use changes::{Changes, POSITION_TOLERANCE, PlayerEvent, detect_changes};
pub use display::{DEFAULT_INPUT_LIMIT, DisplayPolicy, DisplayPower};
pub use playback::{
    ArtworkRef, AudioInput, Content, DeviceSnapshot, InputKind, Origin, Palette, PlaybackState,
    PlayerSnapshot, Position, QueueEntry, QueueView, Rgb, Session, Track,
};
pub use quality::{AudioFormat, AudioQuality, QualityMeasure};
pub use time::Timestamp;
