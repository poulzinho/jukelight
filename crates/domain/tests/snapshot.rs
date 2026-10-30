//! Spec: playback-monitoring, "Player snapshot" and "Queue view".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::time::Duration;

use jukelight_domain::{
    Content, PlaybackState, PlayerSnapshot, Position, QueueEntry, QueueView, Session, Timestamp,
    Track,
};

fn track(title: &str) -> Track {
    Track {
        title: title.into(),
        artists: vec!["Example Artist".into()],
        album: Some("Example Album".into()),
    }
}

fn content(track: Track, position: Option<Position>, queue: QueueView) -> Content {
    Content {
        track,
        artwork: None,
        position,
        quality: None,
        queue,
    }
}

#[test]
fn snapshot_of_a_playing_track() {
    let snapshot = PlayerSnapshot {
        player_name: "Example Player".into(),
        online: true,
        state: PlaybackState::Playing,
        session: Session {
            label: "Qobuz".into(),
            available: true,
        },
        content: Some(content(
            track("Example Song"),
            None,
            QueueView::NotApplicable,
        )),
    };

    assert_eq!(snapshot.state, PlaybackState::Playing);
    let shown = snapshot.content.as_ref().unwrap();
    assert_eq!(shown.track.title, "Example Song");
    assert_eq!(shown.track.artists, vec!["Example Artist".to_owned()]);
    assert_eq!(shown.track.album.as_deref(), Some("Example Album"));
}

#[test]
fn source_without_duration() {
    let position = Position {
        elapsed: Duration::from_secs(10),
        duration: None,
        observed_at: Timestamp::from_millis(0),
    };
    let shown = content(
        track("Example Song"),
        Some(position),
        QueueView::NotApplicable,
    );

    assert_eq!(shown.position.unwrap().duration, None);
    assert_eq!(shown.track.title, "Example Song");
}

#[test]
fn nothing_playing_has_no_content() {
    let snapshot = PlayerSnapshot::nothing_playing("Example Player");

    assert_eq!(snapshot.state, PlaybackState::Stopped);
    assert!(snapshot.content.is_none());
    assert!(!snapshot.online);
}

#[test]
fn session_with_upcoming_tracks() {
    let entries: Vec<QueueEntry> = ["One", "Two", "Three"]
        .into_iter()
        .map(|title| QueueEntry {
            title: title.into(),
            artists: vec![],
            artwork: None,
        })
        .collect();
    let queue = QueueView::Available(entries);

    let QueueView::Available(shown) = &queue else {
        panic!("queue should be available")
    };
    let titles: Vec<_> = shown.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["One", "Two", "Three"]);
}

#[test]
fn session_without_a_queue() {
    let shown = content(track("Example Song"), None, QueueView::NotApplicable);
    assert_eq!(shown.queue, QueueView::NotApplicable);
}
