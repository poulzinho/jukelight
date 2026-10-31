//! Spec: display-control, the decision over time (design decision 1): the
//! controller follows the published player snapshots, with tokio's paused
//! time standing in for real minutes.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::time::Duration;

use jukelight_application::{DisplayController, DisplayHandle, PlayerMonitor};
use jukelight_domain::{
    Content, DeviceSnapshot, DisplayPolicy, DisplayPower, Origin, PlaybackState, QueueView, Track,
};

const MINUTE: Duration = Duration::from_secs(60);

fn device(title: &str, online: bool, state: PlaybackState) -> DeviceSnapshot {
    DeviceSnapshot {
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online,
        state,
        origin: Origin::Service {
            id: "Qobuz".into(),
            name: "Qobuz".into(),
            cast: false,
        },
        content: Some(Content {
            track: Track {
                title: title.into(),
                artists: vec!["Example Artist".into()],
                album: None,
            },
            artwork: None,
            position: None,
            quality: None,
            queue: QueueView::NotApplicable,
            service: None,
            palette: None,
        }),
    }
}

fn playing(title: &str) -> DeviceSnapshot {
    device(title, true, PlaybackState::Playing)
}

fn paused(title: &str) -> DeviceSnapshot {
    device(title, true, PlaybackState::Paused)
}

/// A monitor and a running controller with the given grace period.
fn start(grace: Duration) -> (PlayerMonitor, DisplayHandle) {
    let monitor = PlayerMonitor::new(Vec::new());
    let controller = DisplayController::new(DisplayPolicy::new(grace));
    let display = controller.handle();
    tokio::spawn(controller.run(monitor.handle().subscribe_published()));
    (monitor, display)
}

/// Lets the controller react to what just happened, without moving time
/// beyond `duration`.
async fn wait(duration: Duration) {
    tokio::time::sleep(duration).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test(start_paused = true)]
async fn start_with_nothing_playing() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&device("Example Song", true, PlaybackState::Stopped));
    wait(Duration::ZERO).await;
    assert_eq!(display.current(), DisplayPower::On);

    wait(9 * MINUTE + Duration::from_secs(59)).await;
    assert_eq!(display.current(), DisplayPower::On);
    wait(Duration::from_secs(1)).await;
    assert_eq!(display.current(), DisplayPower::Off);
}

#[tokio::test(start_paused = true)]
async fn start_while_music_plays() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&playing("Example Song"));
    wait(30 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::On);
}

#[tokio::test(start_paused = true)]
async fn paused_past_the_grace_period_then_playback_starts() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&playing("Example Song"));
    wait(MINUTE).await;
    monitor.apply(&paused("Example Song"));
    wait(10 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::Off);

    monitor.apply(&playing("Example Song"));
    wait(Duration::ZERO).await;
    assert_eq!(display.current(), DisplayPower::On);
}

#[tokio::test(start_paused = true)]
async fn playback_resumes_within_the_grace_period() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&playing("Example Song"));
    wait(MINUTE).await;
    monitor.apply(&paused("Example Song"));
    wait(5 * MINUTE).await;
    monitor.apply(&playing("Example Song"));
    wait(MINUTE).await;
    monitor.apply(&paused("Example Song"));

    // Ten minutes after the second pause, not after the first.
    wait(9 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::On);
    wait(MINUTE).await;
    assert_eq!(display.current(), DisplayPower::Off);
}

#[tokio::test(start_paused = true)]
async fn player_offline() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&playing("Example Song"));
    wait(MINUTE).await;
    monitor.apply(&device("Example Song", false, PlaybackState::Playing));
    wait(10 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::Off);
}

#[tokio::test(start_paused = true)]
async fn no_grace_period() {
    let (monitor, display) = start(Duration::ZERO);
    monitor.apply(&playing("Example Song"));
    wait(MINUTE).await;
    monitor.apply(&paused("Example Song"));
    wait(Duration::ZERO).await;
    assert_eq!(display.current(), DisplayPower::Off);
}

#[tokio::test(start_paused = true)]
async fn track_change_while_playing() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&playing("Song A"));
    wait(MINUTE).await;
    let mut decisions = display.subscribe();
    decisions.mark_unchanged();

    monitor.apply(&playing("Song B"));
    wait(MINUTE).await;
    assert!(
        !decisions.has_changed().unwrap(),
        "no new decision expected"
    );
}

#[tokio::test(start_paused = true)]
async fn pause_and_resume_within_the_grace_period() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&playing("Example Song"));
    wait(MINUTE).await;
    let mut decisions = display.subscribe();
    decisions.mark_unchanged();

    monitor.apply(&paused("Example Song"));
    wait(3 * MINUTE).await;
    monitor.apply(&playing("Example Song"));
    wait(20 * MINUTE).await;
    assert!(
        !decisions.has_changed().unwrap(),
        "no new decision expected"
    );
}
