//! Spec: playback-monitoring, "Per-track position".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::time::Duration;

use jukelight_domain::{PlaybackState, Position, Timestamp};

const T: Timestamp = Timestamp::from_millis(1_000_000);

fn at_60s(duration: Option<Duration>) -> Position {
    Position {
        elapsed: Duration::from_secs(60),
        duration,
        observed_at: T,
    }
}

#[test]
fn predicting_progress_while_playing() {
    let position = at_60s(Some(Duration::from_secs(200)));
    let later = T.plus(Duration::from_secs(5));

    assert_eq!(
        position.predict(later, PlaybackState::Playing),
        Duration::from_secs(65)
    );
}

#[test]
fn position_does_not_advance_while_paused() {
    let position = at_60s(Some(Duration::from_secs(200)));
    let later = T.plus(Duration::from_secs(5));

    assert_eq!(
        position.predict(later, PlaybackState::Paused),
        Duration::from_secs(60)
    );
}

#[test]
fn prediction_stops_at_the_end_of_the_track() {
    let position = at_60s(Some(Duration::from_secs(62)));
    let later = T.plus(Duration::from_secs(5));

    assert_eq!(
        position.predict(later, PlaybackState::Playing),
        Duration::from_secs(62)
    );
}

#[test]
fn prediction_without_duration_keeps_counting() {
    let position = at_60s(None);
    let later = T.plus(Duration::from_secs(5));

    assert_eq!(
        position.predict(later, PlaybackState::Playing),
        Duration::from_secs(65)
    );
}

#[test]
fn observation_in_the_past_does_not_go_backwards() {
    let position = at_60s(None);
    let earlier = Timestamp::from_millis(0);

    assert_eq!(
        position.predict(earlier, PlaybackState::Playing),
        Duration::from_secs(60)
    );
}
