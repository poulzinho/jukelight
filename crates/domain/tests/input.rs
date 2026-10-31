//! Spec: playback-monitoring, "Player snapshot" ("Input plays") and "Change
//! detection" for audio inputs.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::time::Duration;

use jukelight_domain::{
    AudioInput, InputKind, PlaybackState, PlayerEvent, PlayerSnapshot, Session, Timestamp,
    detect_changes,
};

const T: Timestamp = Timestamp::from_millis(10_000_000);

fn input(kind: InputKind, name: &str, selected_at: Timestamp) -> AudioInput {
    AudioInput {
        kind,
        name: name.into(),
        selected_at,
    }
}

fn playing_input(input: Option<AudioInput>) -> PlayerSnapshot {
    PlayerSnapshot {
        online: true,
        state: PlaybackState::Playing,
        session: Session {
            label: "Optical input".into(),
            available: true,
        },
        input,
        ..PlayerSnapshot::nothing_playing("Example Player")
    }
}

fn optical(selected_at: Timestamp) -> PlayerSnapshot {
    playing_input(Some(input(
        InputKind::Optical,
        "Example Input",
        selected_at,
    )))
}

#[test]
fn input_plays() {
    let snapshot = optical(T);
    let input = snapshot.input.as_ref().unwrap();
    assert_eq!(input.kind, InputKind::Optical);
    assert_eq!(input.name, "Example Input");
    assert_eq!(input.selected_at, T);
    assert_eq!(snapshot.content, None);
}

#[test]
fn nothing_playing_has_no_input() {
    assert_eq!(
        PlayerSnapshot::nothing_playing("Example Player").input,
        None
    );
}

#[test]
fn input_selected_again() {
    let changes = detect_changes(&optical(T), &optical(T.plus(Duration::from_secs(30 * 60))));
    assert_eq!(changes.events, [PlayerEvent::InputChanged]);
    assert!(changes.material);
}

#[test]
fn selection_time_jitter() {
    for jitter in [1, 2] {
        let changes = detect_changes(&optical(T), &optical(T.plus(Duration::from_secs(jitter))));
        assert!(
            changes.events.is_empty(),
            "{jitter} s: {:?}",
            changes.events
        );
        assert!(!changes.material, "{jitter} s should not be published");
    }
}

#[test]
fn selection_time_moving_back_is_also_a_change() {
    let later = T.plus(Duration::from_secs(600));
    let changes = detect_changes(&optical(later), &optical(T));
    assert_eq!(changes.events, [PlayerEvent::InputChanged]);
}

#[test]
fn renamed_input() {
    let renamed = playing_input(Some(input(InputKind::Optical, "Leak", T)));
    let changes = detect_changes(&optical(T), &renamed);
    assert_eq!(changes.events, [PlayerEvent::InputChanged]);
}

#[test]
fn other_kind() {
    let analog = playing_input(Some(input(InputKind::Analog, "Example Input", T)));
    let changes = detect_changes(&optical(T), &analog);
    assert_eq!(changes.events, [PlayerEvent::InputChanged]);
}

#[test]
fn input_appears_and_goes_away() {
    let none = playing_input(None);
    assert_eq!(
        detect_changes(&none, &optical(T)).events,
        [PlayerEvent::InputChanged]
    );
    assert_eq!(
        detect_changes(&optical(T), &none).events,
        [PlayerEvent::InputChanged]
    );
}
