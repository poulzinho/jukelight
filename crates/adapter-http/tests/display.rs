//! Spec: published-api, "Display decisions".

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use futures::FutureExt;
use futures::future::BoxFuture;
use http_body_util::BodyExt;
use jukelight_adapter_http::{HttpConfig, router_with_shutdown};
use jukelight_application::{
    ArtworkError, ArtworkId, ArtworkSize, ArtworkSource, BrandLogos, DisplayController, Image,
    PlayerMonitor, SystemClock,
};
use jukelight_domain::{ArtworkRef, Brand, DeviceSnapshot, DisplayPolicy, Origin, PlaybackState};
use tokio::sync::watch;
use tower::ServiceExt;

struct NoArtwork;

struct NoLogos;

impl BrandLogos for NoLogos {
    fn version(&self, _: &Brand) -> Option<u64> {
        None
    }

    fn logo(&self, _: &Brand) -> Option<Image> {
        None
    }
}

impl ArtworkSource for NoArtwork {
    fn issue(&self, _: &ArtworkRef) -> ArtworkId {
        ArtworkId::new("none")
    }
    fn fetch(
        &self,
        _: &ArtworkId,
        _: ArtworkSize,
    ) -> BoxFuture<'static, Result<Image, ArtworkError>> {
        async { Err(ArtworkError::Unknown) }.boxed()
    }
}

fn device(state: PlaybackState) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state,
        origin: Origin::None,
        content: None,
    }
}

/// A playing player, a display controller with no grace period, and the
/// router; `stop` requests shutdown.
struct Setup {
    monitor: PlayerMonitor,
    app: Router,
    stop: watch::Sender<bool>,
}

fn setup() -> Setup {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device(PlaybackState::Playing));
    let controller =
        DisplayController::new(DisplayPolicy::new(Duration::ZERO), Arc::new(SystemClock));
    let display = controller.handle();
    tokio::spawn(controller.run(monitor.handle().subscribe_published()));
    let (stop, shutdown) = watch::channel(false);
    let config = HttpConfig {
        keep_alive: Duration::from_millis(100),
    };
    let app = router_with_shutdown(
        monitor.handle(),
        display,
        Arc::new(NoArtwork),
        Arc::new(NoLogos),
        &config,
        shutdown,
    );
    Setup { monitor, app, stop }
}

async fn open(app: &Router) -> Body {
    let response = app
        .clone()
        .oneshot(Request::get("/api/v1/display").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    response.into_body()
}

/// Reads SSE lines until `deadline`: the `display` events' data, and the
/// number of comment lines.
async fn read(body: &mut Body, deadline: Duration) -> (Vec<String>, usize) {
    let mut decisions = Vec::new();
    let mut comments = 0;
    let mut event = String::new();
    let _ = tokio::time::timeout(deadline, async {
        while let Some(Ok(frame)) = body.frame().await {
            let Some(chunk) = frame.data_ref() else {
                continue;
            };
            for line in String::from_utf8_lossy(chunk).lines() {
                if let Some(name) = line.strip_prefix("event:") {
                    name.trim().clone_into(&mut event);
                } else if let Some(data) = line.strip_prefix("data:") {
                    assert_eq!(event, "display", "events are named display");
                    decisions.push(data.trim().to_owned());
                } else if line.starts_with(':') {
                    comments += 1;
                }
            }
        }
    })
    .await;
    (decisions, comments)
}

#[tokio::test]
async fn connect() {
    let setup = setup();
    let mut body = open(&setup.app).await;
    let (decisions, _) = read(&mut body, Duration::from_millis(50)).await;
    assert_eq!(decisions, ["on"]);
}

#[tokio::test]
async fn decision_changes() {
    let setup = setup();
    let mut body = open(&setup.app).await;
    let _ = read(&mut body, Duration::from_millis(50)).await;

    setup.monitor.apply(&device(PlaybackState::Paused));
    let (decisions, _) = read(&mut body, Duration::from_millis(200)).await;
    assert_eq!(decisions, ["off"]);

    setup.monitor.apply(&device(PlaybackState::Playing));
    let (decisions, _) = read(&mut body, Duration::from_millis(200)).await;
    assert_eq!(decisions, ["on"]);
}

#[tokio::test]
async fn heartbeat() {
    let setup = setup();
    let mut body = open(&setup.app).await;
    let (decisions, comments) = read(&mut body, Duration::from_millis(350)).await;
    assert_eq!(decisions, ["on"], "no new decision without a change");
    assert!(comments >= 2, "comment lines keep the stream alive");
}

#[test]
fn default_heartbeat_is_within_30_seconds() {
    assert!(HttpConfig::default().keep_alive <= Duration::from_secs(30));
}

#[tokio::test]
async fn shutdown() {
    let setup = setup();
    let mut body = open(&setup.app).await;
    let _ = read(&mut body, Duration::from_millis(50)).await;

    setup.stop.send(true).unwrap();
    let ended = tokio::time::timeout(Duration::from_secs(1), async {
        while let Some(frame) = body.frame().await {
            frame.unwrap();
        }
    })
    .await;
    assert!(ended.is_ok(), "the display stream should end on shutdown");
}
