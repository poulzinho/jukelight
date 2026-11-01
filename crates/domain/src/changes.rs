use std::time::Duration;

use crate::{PlaybackState, PlayerSnapshot, Timestamp};

/// How far the reported elapsed time may differ from the predicted one
/// before it counts as a jump (a seek), not normal progress.
pub const POSITION_TOLERANCE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerEvent {
    TrackChanged,
    PlaybackStateChanged {
        from: PlaybackState,
        to: PlaybackState,
    },
    QueueChanged,
    PositionResynced,
    /// An input appeared or went away, its kind or name changed, its signal
    /// started, or its start time moved.
    InputChanged,
    PlayerWentOffline,
    PlayerCameOnline,
}

/// What changed between two snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Changes {
    pub events: Vec<PlayerEvent>,
    /// True when anything changed other than progress that matches the
    /// prediction. Clients only need a new snapshot when this is true.
    pub material: bool,
}

pub fn detect_changes(previous: &PlayerSnapshot, next: &PlayerSnapshot) -> Changes {
    let mut events = Vec::new();

    if previous.online && !next.online {
        events.push(PlayerEvent::PlayerWentOffline);
    }
    if !previous.online && next.online {
        events.push(PlayerEvent::PlayerCameOnline);
    }

    let previous_track = previous.content.as_ref().map(|c| &c.track);
    let next_track = next.content.as_ref().map(|c| &c.track);
    let same_track = previous_track == next_track;
    if !same_track {
        events.push(PlayerEvent::TrackChanged);
    }

    if previous.state != next.state {
        events.push(PlayerEvent::PlaybackStateChanged {
            from: previous.state,
            to: next.state,
        });
    }

    let previous_queue = previous.content.as_ref().map(|c| &c.queue);
    let next_queue = next.content.as_ref().map(|c| &c.queue);
    if same_track && previous_queue != next_queue {
        events.push(PlayerEvent::QueueChanged);
    }

    if same_track && jumped(previous, next) {
        events.push(PlayerEvent::PositionResynced);
    }

    if input_changed(previous, next) {
        events.push(PlayerEvent::InputChanged);
    }

    let material = !events.is_empty() || without_progress(previous) != without_progress(next);
    Changes { events, material }
}

/// True when the new elapsed time is too far from the predicted one.
fn jumped(previous: &PlayerSnapshot, next: &PlayerSnapshot) -> bool {
    let positions = (
        previous.content.as_ref().and_then(|c| c.position),
        next.content.as_ref().and_then(|c| c.position),
    );
    let (Some(before), Some(after)) = positions else {
        return false;
    };
    let predicted = before.predict(after.observed_at, previous.state);
    predicted.abs_diff(after.elapsed) > POSITION_TOLERANCE
}

/// True when an input appeared or went away, its kind or name changed, its
/// signal started, or its start time moved by more than the tolerance (the
/// player reports it in whole seconds, so each report shifts it slightly).
fn input_changed(previous: &PlayerSnapshot, next: &PlayerSnapshot) -> bool {
    match (&previous.input, &next.input) {
        (None, None) => false,
        (Some(before), Some(after)) => {
            before.kind != after.kind
                || before.name != after.name
                || match (before.started_at, after.started_at) {
                    (None, None) => false,
                    (Some(a), Some(b)) => {
                        a.since(b) > POSITION_TOLERANCE || b.since(a) > POSITION_TOLERANCE
                    }
                    _ => true,
                }
        }
        _ => true,
    }
}

/// The snapshot with its moving parts (elapsed time, observation time and
/// an input's start time) cleared, so normal progress compares equal.
fn without_progress(snapshot: &PlayerSnapshot) -> PlayerSnapshot {
    let mut copy = snapshot.clone();
    if let Some(position) = copy.content.as_mut().and_then(|c| c.position.as_mut()) {
        position.elapsed = Duration::ZERO;
        position.observed_at = Timestamp::default();
    }
    if let Some(started_at) = copy.input.as_mut().and_then(|i| i.started_at.as_mut()) {
        *started_at = Timestamp::default();
    }
    copy
}
