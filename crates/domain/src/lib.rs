//! Jukelight's playback model, in its own language.
//!
//! What a player is doing (a snapshot), how its position moves, how good the
//! audio is, and which events follow from one snapshot to the next. No IO,
//! no vendor terms: adapters translate into these types.

mod changes;
mod playback;
mod quality;
mod time;

pub use changes::{Changes, POSITION_TOLERANCE, PlayerEvent, detect_changes};
pub use playback::{
    ArtworkRef, Content, DeviceSnapshot, Origin, Palette, PlaybackState, PlayerSnapshot, Position,
    QueueEntry, QueueView, Rgb, Session, Track,
};
pub use quality::{AudioFormat, AudioQuality, QualityMeasure};
pub use time::Timestamp;
