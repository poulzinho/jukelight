//! Spec: display-control, "Display on while music plays" and "Display off
//! after the grace period" (the policy; timing is in the application).

use std::time::Duration;

use jukelight_domain::{DisplayPolicy, DisplayPower, PlaybackState, PlayerSnapshot};

const MINUTE: Duration = Duration::from_secs(60);

fn player(online: bool, state: PlaybackState) -> PlayerSnapshot {
    PlayerSnapshot {
        online,
        state,
        ..PlayerSnapshot::nothing_playing("Example Player")
    }
}

fn policy() -> DisplayPolicy {
    DisplayPolicy::new(10 * MINUTE)
}

#[test]
fn playback_starts() {
    assert!(DisplayPolicy::is_active(&player(
        true,
        PlaybackState::Playing
    )));
    assert_eq!(policy().decide(None), DisplayPower::On);
}

#[test]
fn buffering() {
    assert!(DisplayPolicy::is_active(&player(
        true,
        PlaybackState::Buffering
    )));
}

#[test]
fn paused_within_the_grace_period() {
    assert!(!DisplayPolicy::is_active(&player(
        true,
        PlaybackState::Paused
    )));
    assert_eq!(policy().decide(Some(9 * MINUTE)), DisplayPower::On);
}

#[test]
fn paused_past_the_grace_period() {
    assert_eq!(policy().decide(Some(10 * MINUTE)), DisplayPower::Off);
    assert_eq!(policy().decide(Some(11 * MINUTE)), DisplayPower::Off);
}

#[test]
fn stopped() {
    assert!(!DisplayPolicy::is_active(&player(
        true,
        PlaybackState::Stopped
    )));
}

#[test]
fn player_offline() {
    // Offline counts as not playing, whatever state was last reported.
    assert!(!DisplayPolicy::is_active(&player(
        false,
        PlaybackState::Playing
    )));
    assert!(!DisplayPolicy::is_active(&player(
        false,
        PlaybackState::Buffering
    )));
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
