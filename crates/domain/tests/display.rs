//! Spec: display-control, "Display on while music plays" and "Display off
//! after the grace period" (the policy; timing is in the application).

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::time::Duration;

use jukelight_domain::{
    AudioInput, DisplayPolicy, DisplayPower, InputKind, PlaybackState, PlayerSnapshot, Timestamp,
};

const MINUTE: Duration = Duration::from_secs(60);
const NOW: Timestamp = Timestamp::from_millis(100_000_000);

fn player(online: bool, state: PlaybackState) -> PlayerSnapshot {
    PlayerSnapshot {
        online,
        state,
        ..PlayerSnapshot::nothing_playing("Example Player")
    }
}

fn policy() -> DisplayPolicy {
    DisplayPolicy::new(10 * MINUTE).with_input_limit(90 * MINUTE)
}

#[test]
fn playback_starts() {
    assert!(policy().is_active(&player(true, PlaybackState::Playing), NOW));
    assert_eq!(policy().decide(None), DisplayPower::On);
}

#[test]
fn buffering() {
    assert!(policy().is_active(&player(true, PlaybackState::Buffering), NOW));
}

#[test]
fn paused_within_the_grace_period() {
    assert!(!policy().is_active(&player(true, PlaybackState::Paused), NOW));
    assert_eq!(policy().decide(Some(9 * MINUTE)), DisplayPower::On);
}

#[test]
fn paused_past_the_grace_period() {
    assert_eq!(policy().decide(Some(10 * MINUTE)), DisplayPower::Off);
    assert_eq!(policy().decide(Some(11 * MINUTE)), DisplayPower::Off);
}

#[test]
fn stopped() {
    assert!(!policy().is_active(&player(true, PlaybackState::Stopped), NOW));
}

#[test]
fn player_offline() {
    // Offline counts as not playing, whatever state was last reported.
    assert!(!policy().is_active(&player(false, PlaybackState::Playing), NOW));
    assert!(!policy().is_active(&player(false, PlaybackState::Buffering), NOW));
}

#[test]
fn no_grace_period() {
    let policy = DisplayPolicy::new(Duration::ZERO);
    assert_eq!(policy.decide(Some(Duration::ZERO)), DisplayPower::Off);
    assert_eq!(policy.decide(None), DisplayPower::On);
}

#[test]
fn remaining_time_until_off() {
    assert_eq!(policy().remaining(Some(4 * MINUTE)), Some(6 * MINUTE));
    assert_eq!(policy().remaining(Some(12 * MINUTE)), Some(Duration::ZERO));
    assert_eq!(policy().remaining(None), None);
}

// Spec: display-control, "Inputs count as playing for a limited time".

fn input_selected(ago: Duration) -> PlayerSnapshot {
    PlayerSnapshot {
        input: Some(AudioInput {
            kind: InputKind::Optical,
            name: "Example Input".into(),
            selected_at: Timestamp::from_millis(
                NOW.as_millis() - u64::try_from(ago.as_millis()).unwrap(),
            ),
        }),
        ..player(true, PlaybackState::Playing)
    }
}

#[test]
fn within_the_input_time_limit() {
    let player = input_selected(80 * MINUTE);
    assert!(policy().is_active(&player, NOW));
    assert_eq!(policy().input_time_left(&player, NOW), Some(10 * MINUTE));
}

#[test]
fn past_the_input_time_limit() {
    let player = input_selected(100 * MINUTE);
    assert!(!policy().is_active(&player, NOW));
    assert_eq!(policy().input_time_left(&player, NOW), Some(Duration::ZERO));
    // Inactive since the limit passed, 10 minutes ago: past the grace period.
    assert_eq!(policy().decide(Some(10 * MINUTE)), DisplayPower::Off);
}

#[test]
fn input_limit_of_zero() {
    let policy = DisplayPolicy::new(10 * MINUTE).with_input_limit(Duration::ZERO);
    assert!(!policy.is_active(&input_selected(Duration::ZERO), NOW));
}

#[test]
fn another_source_has_no_time_limit() {
    let player = player(true, PlaybackState::Playing);
    assert!(policy().is_active(&player, NOW));
    assert_eq!(policy().input_time_left(&player, NOW), None);
}

#[test]
fn paused_input_is_not_active() {
    let player = PlayerSnapshot {
        state: PlaybackState::Paused,
        ..input_selected(MINUTE)
    };
    assert!(!policy().is_active(&player, NOW));
}
