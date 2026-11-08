//! Spec: display-control, the decision over time (design decision 1): the
//! controller follows the published player snapshots, with tokio's paused
//! time standing in for real minutes.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;
use std::time::Duration;

use jukelight_application::{Clock, DisplayController, DisplayHandle, PlayerMonitor};
use jukelight_domain::{
    AudioInput, Content, DeviceSnapshot, DisplayPolicy, DisplayPower, InputKind, Origin,
    PlaybackState, QueueView, Timestamp, Track,
};

const MINUTE: Duration = Duration::from_secs(60);

fn device(title: &str, online: bool, state: PlaybackState) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online,
        state,
        origin: Origin::Service {
            brand: None,
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
            delivery: None,
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

/// Wall time that moves with tokio's (paused) time.
struct TokioClock {
    start: tokio::time::Instant,
}

/// The wall time when a test starts.
const START: Timestamp = Timestamp::from_millis(1_000_000_000);

impl Clock for TokioClock {
    fn now(&self) -> Timestamp {
        START.plus(self.start.elapsed())
    }
}

/// A monitor and a running controller with the given grace period.
fn start(grace: Duration) -> (PlayerMonitor, DisplayHandle) {
    start_with_input_limit(grace, 90 * MINUTE)
}

fn start_with_input_limit(
    grace: Duration,
    input_limit: Duration,
) -> (PlayerMonitor, DisplayHandle) {
    let monitor = PlayerMonitor::new(Vec::new());
    let clock = Arc::new(TokioClock {
        start: tokio::time::Instant::now(),
    });
    let controller = DisplayController::new(
        DisplayPolicy::new(grace).with_input_limit(input_limit),
        clock,
    );
    let display = controller.handle();
    tokio::spawn(controller.run(monitor.handle().subscribe_published()));
    (monitor, display)
}

/// The player playing its optical input, its signal started at `started_at`.
fn optical_input(started_at: Timestamp) -> DeviceSnapshot {
    optical(Some(started_at), PlaybackState::Playing)
}

/// The optical input, waiting for a signal.
fn waiting_input() -> DeviceSnapshot {
    optical(None, PlaybackState::Buffering)
}

fn optical(started_at: Option<Timestamp>, state: PlaybackState) -> DeviceSnapshot {
    DeviceSnapshot {
        origin: Origin::Input(AudioInput {
            kind: InputKind::Optical,
            name: "Example Input".into(),
            started_at,
        }),
        state,
        content: None,
        ..playing("unused")
    }
}

fn before_start(ago: Duration) -> Timestamp {
    Timestamp::from_millis(START.as_millis() - u64::try_from(ago.as_millis()).unwrap())
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

// Spec: display-control, "Inputs count as playing for a limited time".

#[tokio::test(start_paused = true)]
async fn within_the_input_time_limit() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&optical_input(before_start(80 * MINUTE)));
    wait(9 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::On);
}

#[tokio::test(start_paused = true)]
async fn past_the_limit_and_the_grace_period() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&optical_input(before_start(80 * MINUTE)));
    // The limit passes after 10 minutes, the grace period 10 minutes later.
    wait(19 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::On);
    wait(MINUTE).await;
    assert_eq!(display.current(), DisplayPower::Off);
}

#[tokio::test(start_paused = true)]
async fn start_long_after_the_input_was_selected() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&optical_input(before_start(100 * MINUTE)));
    wait(Duration::ZERO).await;
    assert_eq!(display.current(), DisplayPower::Off);
}

#[tokio::test(start_paused = true)]
async fn signal_starts_again() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&optical_input(before_start(100 * MINUTE)));
    wait(MINUTE).await;
    assert_eq!(display.current(), DisplayPower::Off);

    // The signal starts again, one minute after the test's start.
    monitor.apply(&optical_input(START.plus(MINUTE)));
    wait(Duration::ZERO).await;
    assert_eq!(display.current(), DisplayPower::On);
}

#[tokio::test(start_paused = true)]
async fn another_source() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&playing("Example Song"));
    wait(180 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::On);
}

#[tokio::test(start_paused = true)]
async fn input_limit_of_zero() {
    let (monitor, display) = start_with_input_limit(Duration::ZERO, Duration::ZERO);
    monitor.apply(&optical_input(START));
    wait(Duration::ZERO).await;
    assert_eq!(display.current(), DisplayPower::Off);
}

#[tokio::test(start_paused = true)]
async fn waiting_for_a_signal() {
    let (monitor, display) = start(10 * MINUTE);
    monitor.apply(&waiting_input());
    wait(9 * MINUTE).await;
    assert_eq!(display.current(), DisplayPower::On);
    wait(MINUTE).await;
    assert_eq!(display.current(), DisplayPower::Off);

    // The CD starts.
    monitor.apply(&optical_input(START.plus(10 * MINUTE)));
    wait(Duration::ZERO).await;
    assert_eq!(display.current(), DisplayPower::On);
}
