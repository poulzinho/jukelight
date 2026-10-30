//! Spec: device-bluos, "Long-polling" and "Connectivity and reconnection",
//! against a local test server that replays recorded responses.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::{RawQuery, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use futures::StreamExt;
use jukelight_adapter_bluos::{BluosConfig, BluosMonitor, Timing};
use jukelight_application::DeviceMonitor;
use jukelight_application::testing::FixedClock;
use jukelight_application::testing::contract::{DeviceHarness, device_monitor_contract};

/// A fake BluOS player serving one recorded situation.
struct Player {
    fixture: String,
    reachable: bool,
    /// How long a long-poll with an unchanged etag is held open.
    hold: Duration,
    requests: Vec<(Instant, String)>,
}

#[derive(Clone)]
struct TestPlayer(Arc<Mutex<Player>>);

impl TestPlayer {
    async fn start(fixture: &str, hold: Duration) -> (Self, u16) {
        let player = TestPlayer(Arc::new(Mutex::new(Player {
            fixture: fixture.into(),
            reachable: true,
            hold,
            requests: Vec::new(),
        })));
        let app = Router::new()
            .route("/Status", get(status))
            .route(
                "/SyncStatus",
                get(|s: State<TestPlayer>| async move { file(s, "SyncStatus", "syncstatus.xml") }),
            )
            .route(
                "/Playlist",
                get(|s: State<TestPlayer>| async move { file(s, "Playlist", "playlist.xml") }),
            )
            .with_state(player.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (player, port)
    }

    fn set_reachable(&self, reachable: bool) {
        self.0.lock().unwrap().reachable = reachable;
    }

    fn requests(&self) -> Vec<(Instant, String)> {
        self.0.lock().unwrap().requests.clone()
    }
}

impl DeviceHarness for TestPlayer {
    fn make_unreachable(&self) {
        self.set_reachable(false);
    }
    fn make_reachable(&self) {
        self.set_reachable(true);
    }
}

fn fixture_path(fixture: &str, file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture)
        .join(file)
}

/// Records the request; `None` when the player is unreachable.
fn record(player: &TestPlayer, request: String) -> Option<(String, Duration)> {
    let mut state = player.0.lock().unwrap();
    state.requests.push((Instant::now(), request));
    state.reachable.then(|| (state.fixture.clone(), state.hold))
}

async fn status(State(player): State<TestPlayer>, RawQuery(query): RawQuery) -> impl IntoResponse {
    let Some((fixture, hold)) = record(
        &player,
        format!("Status?{}", query.clone().unwrap_or_default()),
    ) else {
        return (StatusCode::SERVICE_UNAVAILABLE, String::new());
    };
    let body = std::fs::read_to_string(fixture_path(&fixture, "status.xml")).unwrap();
    let etag = body
        .split("etag=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or("");
    let unchanged = query.is_some_and(|q| q.contains(&format!("etag={etag}")));
    if unchanged {
        tokio::time::sleep(hold).await;
    }
    (StatusCode::OK, body)
}

fn file(State(player): State<TestPlayer>, request: &str, name: &str) -> impl IntoResponse {
    let Some((fixture, _)) = record(&player, request.into()) else {
        return (StatusCode::SERVICE_UNAVAILABLE, String::new());
    };
    (
        StatusCode::OK,
        std::fs::read_to_string(fixture_path(&fixture, name)).unwrap(),
    )
}

fn monitor(port: u16, timing: Timing) -> BluosMonitor {
    BluosMonitor::new(
        &BluosConfig {
            host: "127.0.0.1".into(),
            port,
        },
        Arc::new(FixedClock::at(0)),
    )
    .with_timing(timing)
}

fn fast_timing() -> Timing {
    Timing {
        long_poll: Duration::from_secs(1),
        min_spacing: Duration::from_millis(100),
        backoff_start: Duration::from_millis(50),
        backoff_max: Duration::from_millis(200),
        request_timeout: Duration::from_secs(2),
    }
}

#[test]
fn default_timing_follows_the_spec() {
    let timing = Timing::default();
    assert_eq!(timing.long_poll, Duration::from_secs(100));
    assert_eq!(timing.min_spacing, Duration::from_secs(1));
    assert_eq!(timing.backoff_max, Duration::from_secs(30));
}

#[tokio::test]
async fn device_monitor_contract_holds() {
    let (player, port) =
        TestPlayer::start("qobuz-native-playing", Duration::from_millis(200)).await;
    let monitor = monitor(port, fast_timing());
    device_monitor_contract(&monitor, &player).await;
}

#[tokio::test]
async fn timeout_without_changes_keeps_the_device_online() {
    let (player, port) =
        TestPlayer::start("qobuz-native-playing", Duration::from_millis(300)).await;
    let mut snapshots = monitor(port, fast_timing()).snapshots();

    let first = snapshots.next().await.unwrap();
    assert!(first.online);
    tokio::time::sleep(Duration::from_millis(900)).await;

    let status_requests: Vec<_> = player
        .requests()
        .into_iter()
        .filter(|(_, r)| r.starts_with("Status"))
        .collect();
    assert!(
        status_requests.len() >= 2,
        "expected several long-polls: {status_requests:?}"
    );
    assert!(
        status_requests[1..]
            .iter()
            .all(|(_, r)| r.contains("timeout=1") && r.contains("etag=")),
        "follow-up requests must long-poll with the etag: {status_requests:?}"
    );
    let latest = tokio::time::timeout(Duration::from_millis(500), snapshots.next()).await;
    if let Ok(Some(snapshot)) = latest {
        assert!(
            snapshot.online,
            "a long-poll timeout must not mark the device offline"
        );
    }
}

#[tokio::test]
async fn request_spacing() {
    // Hold 0: every long-poll returns at once, so spacing is the only limit.
    let (player, port) = TestPlayer::start("qobuz-native-playing", Duration::ZERO).await;
    let mut snapshots = monitor(port, fast_timing()).snapshots();
    snapshots.next().await.unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;

    let times: Vec<Instant> = player
        .requests()
        .into_iter()
        .filter(|(_, r)| r.starts_with("Status"))
        .map(|(t, _)| t)
        .collect();
    assert!(
        times.len() >= 3,
        "expected several requests, got {}",
        times.len()
    );
    for pair in times.windows(2) {
        let gap = pair[1] - pair[0];
        assert!(
            gap >= Duration::from_millis(95),
            "requests only {gap:?} apart"
        );
    }
}

#[tokio::test]
async fn offline_on_failure_with_capped_backoff() {
    let (player, port) = TestPlayer::start("qobuz-native-playing", Duration::ZERO).await;
    player.set_reachable(false);
    let mut snapshots = monitor(port, fast_timing()).snapshots();

    let first = snapshots.next().await.unwrap();
    assert!(!first.online);
    tokio::time::sleep(Duration::from_millis(1200)).await;

    let times: Vec<Instant> = player.requests().into_iter().map(|(t, _)| t).collect();
    let gaps: Vec<Duration> = times.windows(2).map(|p| p[1] - p[0]).collect();
    assert!(gaps.len() >= 4, "expected several attempts: {gaps:?}");
    // Delays grow (50, 100, 200 ms) and never pass the 200 ms cap.
    assert!(gaps[1] > gaps[0], "delay should grow: {gaps:?}");
    assert!(
        gaps.iter().all(|g| *g < Duration::from_millis(300)),
        "delay above the cap: {gaps:?}"
    );
    assert!(
        gaps.last().unwrap() >= &Duration::from_millis(180),
        "delay should reach the cap: {gaps:?}"
    );
}
