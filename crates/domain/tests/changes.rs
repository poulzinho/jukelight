//! Spec: playback-monitoring, "Change detection".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::time::Duration;

use jukelight_domain::{
    Content, PlaybackState, PlayerEvent, PlayerSnapshot, Position, QueueEntry, QueueView, Session,
    Timestamp, Track, detect_changes,
};

const T: Timestamp = Timestamp::from_millis(1_000_000);

fn playing(title: &str, elapsed_s: u64, at: Timestamp) -> PlayerSnapshot {
    PlayerSnapshot {
        player_name: "Example Player".into(),
        online: true,
        state: PlaybackState::Playing,
        session: Session {
            label: "Qobuz".into(),
            available: true,
        },
        content: Some(Content {
            track: Track {
                title: title.into(),
                artists: vec!["Example Artist".into()],
                album: None,
            },
            artwork: None,
            position: Some(Position {
                elapsed: Duration::from_secs(elapsed_s),
                duration: Some(Duration::from_secs(300)),
                observed_at: at,
            }),
            quality: None,
            queue: QueueView::NotApplicable,
            service: None,
            palette: None,
        }),
    }
}

fn secs(s: u64) -> Duration {
    Duration::from_secs(s)
}

#[test]
fn new_track() {
    let changes = detect_changes(
        &playing("Song A", 60, T),
        &playing("Song B", 0, T.plus(secs(1))),
    );

    assert!(changes.events.contains(&PlayerEvent::TrackChanged));
    assert!(changes.material);
}

#[test]
fn normal_progress_produces_no_event() {
    let changes = detect_changes(
        &playing("Song A", 60, T),
        &playing("Song A", 90, T.plus(secs(30))),
    );

    assert_eq!(changes.events, Vec::new());
    assert!(!changes.material);
}

#[test]
fn small_drift_is_tolerated() {
    let changes = detect_changes(
        &playing("Song A", 60, T),
        &playing("Song A", 92, T.plus(secs(30))),
    );

    assert_eq!(changes.events, Vec::new());
}

#[test]
fn seek() {
    let changes = detect_changes(
        &playing("Song A", 60, T),
        &playing("Song A", 150, T.plus(secs(10))),
    );

    assert_eq!(changes.events, vec![PlayerEvent::PositionResynced]);
    assert!(changes.material);
}

#[test]
fn pause() {
    let previous = playing("Song A", 60, T);
    let mut next = playing("Song A", 61, T.plus(secs(1)));
    next.state = PlaybackState::Paused;

    let changes = detect_changes(&previous, &next);

    assert!(changes.events.contains(&PlayerEvent::PlaybackStateChanged {
        from: PlaybackState::Playing,
        to: PlaybackState::Paused,
    }));
}

#[test]
fn player_goes_offline() {
    let previous = playing("Song A", 60, T);
    let mut next = previous.clone();
    next.online = false;

    assert!(
        detect_changes(&previous, &next)
            .events
            .contains(&PlayerEvent::PlayerWentOffline)
    );
}

#[test]
fn player_comes_back_online() {
    let mut previous = playing("Song A", 60, T);
    previous.online = false;
    let next = playing("Song A", 60, T);

    assert!(
        detect_changes(&previous, &next)
            .events
            .contains(&PlayerEvent::PlayerCameOnline)
    );
}

#[test]
fn queue_change() {
    let previous = playing("Song A", 60, T);
    let mut next = previous.clone();
    next.content.as_mut().unwrap().queue = QueueView::Available(vec![QueueEntry {
        title: "Song B".into(),
        artists: vec![],
        artwork: None,
    }]);

    assert!(
        detect_changes(&previous, &next)
            .events
            .contains(&PlayerEvent::QueueChanged)
    );
}

#[test]
fn playback_starts_from_nothing() {
    let previous = PlayerSnapshot::nothing_playing("Example Player");
    let next = playing("Song A", 0, T);

    let changes = detect_changes(&previous, &next);
    assert!(changes.events.contains(&PlayerEvent::TrackChanged));
    assert!(changes.events.contains(&PlayerEvent::PlayerCameOnline));
}

#[test]
fn other_field_changes_are_material_without_an_event() {
    let previous = playing("Song A", 60, T);
    let mut next = previous.clone();
    next.session.label = "Spotify".into();

    let changes = detect_changes(&previous, &next);
    assert_eq!(changes.events, Vec::new());
    assert!(changes.material);
}
