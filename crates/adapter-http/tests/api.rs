//! Spec: published-api, plus the HTTP side of the artwork spec.

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use futures::FutureExt;
use futures::future::BoxFuture;
use http_body_util::BodyExt;
use jukelight_adapter_http::{HttpConfig, router, router_with_shutdown};
use jukelight_application::testing::FakeProvider;
use jukelight_application::{
    ArtworkError, ArtworkId, ArtworkSize, ArtworkSource, BrandLogos, DisplayController,
    DisplayHandle, Image, PlayerMonitor,
};
use jukelight_domain::{
    ArtworkRef, AudioFormat, AudioInput, AudioQuality, Brand, Change, Content, Decibels, Delivery,
    DeviceSnapshot, InputKind, Origin, Palette, PlaybackState, Position, QueueEntry, QueueView,
    Rgb, Service, Timestamp, Track,
};
use serde_json::Value;
use tower::ServiceExt;

/// Serves one fixed image for the id "known"; any other id is unknown, and
/// "broken" fails like an unreachable source.
struct TestArtwork;

impl ArtworkSource for TestArtwork {
    fn issue(&self, artwork: &ArtworkRef) -> ArtworkId {
        ArtworkId::new(if artwork.source().contains("broken") {
            "broken"
        } else {
            "known"
        })
    }

    fn fetch(
        &self,
        id: &ArtworkId,
        size: ArtworkSize,
    ) -> BoxFuture<'static, Result<Image, ArtworkError>> {
        let result = match id.as_str() {
            // Each size is told apart by its bytes.
            "known" => Ok(Image {
                content_type: "image/jpeg".into(),
                bytes: match size {
                    ArtworkSize::Original => vec![0xFF, 0xD8],
                    ArtworkSize::Large => vec![0xFF, 0xD8, 0x01],
                    ArtworkSize::Backdrop => vec![0xFF, 0xD8, 0x02],
                },
            }),
            "broken" => Err(ArtworkError::Source("unreachable".into())),
            _ => Err(ArtworkError::Unknown),
        };
        async move { result }.boxed()
    }
}

fn device(title: &str, elapsed_s: u64, at_ms: u64) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
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
                album: Some("Example Album".into()),
            },
            artwork: Some(ArtworkRef::new(
                "http://192.168.1.40:11000/Artwork?songid=1",
            )),
            position: Some(Position {
                elapsed: Duration::from_secs(elapsed_s),
                duration: Some(Duration::from_secs(214)),
                observed_at: Timestamp::from_millis(at_ms),
            }),
            quality: Some(
                AudioQuality::format(AudioFormat {
                    codec: "FLAC".into(),
                    bit_depth: 24,
                    sample_rate_hz: 96_000,
                })
                .with_detail("Hi-Res"),
            ),
            delivery: None,
            queue: QueueView::Available(vec![QueueEntry {
                title: "Next Song".into(),
                artists: vec!["Example Artist".into()],
                artwork: Some(ArtworkRef::new(
                    "http://192.168.1.40:11000/Artwork?songid=2",
                )),
            }]),
            service: None,
            palette: None,
            speakers: Vec::new(),
            held: false,
        }),
        outputs: Vec::new(),
    }
}

/// A display decision that stays on; the display stream has its own tests.
fn display() -> DisplayHandle {
    DisplayController::new(
        jukelight_domain::DisplayPolicy::new(Duration::from_secs(600)),
        Arc::new(jukelight_application::SystemClock),
    )
    .handle()
}

/// Logos the test supplies, by brand, each with a version.
#[derive(Default)]
struct TestLogos(std::sync::Mutex<std::collections::HashMap<String, (u64, Image)>>);

impl TestLogos {
    fn supply(&self, brand: &str, version: u64, content_type: &str, bytes: &[u8]) {
        self.0.lock().unwrap().insert(
            brand.into(),
            (
                version,
                Image {
                    content_type: content_type.into(),
                    bytes: bytes.to_vec(),
                },
            ),
        );
    }
}

impl BrandLogos for TestLogos {
    fn version(&self, brand: &Brand) -> Option<u64> {
        self.0.lock().unwrap().get(brand.as_str()).map(|(v, _)| *v)
    }

    fn logo(&self, brand: &Brand) -> Option<Image> {
        self.0
            .lock()
            .unwrap()
            .get(brand.as_str())
            .map(|(_, image)| image.clone())
    }
}

fn app(monitor: &PlayerMonitor) -> Router {
    app_with_logos(monitor, Arc::new(TestLogos::default()))
}

fn app_with_logos(monitor: &PlayerMonitor, logos: Arc<TestLogos>) -> Router {
    let config = HttpConfig {
        keep_alive: Duration::from_millis(100),
    };
    router(
        monitor.handle(),
        display(),
        Arc::new(TestArtwork),
        logos,
        &config,
    )
}

async fn get(app: &Router, path: &str) -> (StatusCode, Vec<u8>, Option<String>) {
    let response = app
        .clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_owned());
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body, content_type)
}

async fn player_json(app: &Router) -> Value {
    let (status, body, _) = get(app, "/api/v1/player").await;
    assert_eq!(status, StatusCode::OK);
    serde_json::from_slice(&body).unwrap()
}

// 7.1 Current snapshot

#[tokio::test]
async fn playing_track() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));

    let json = player_json(&app(&monitor)).await;
    assert_eq!(json["state"], "playing");
    assert_eq!(json["track"]["title"], "Example Song");
}

#[tokio::test]
async fn nothing_playing() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut stopped = device("x", 0, 0);
    stopped.state = PlaybackState::Stopped;
    stopped.content = None;
    monitor.apply(&stopped);

    let json = player_json(&app(&monitor)).await;
    assert_eq!(json["state"], "stopped");
    assert!(json["track"].is_null());
    assert!(json["artUrl"].is_null());
    assert!(json["backdropUrl"].is_null());
    assert!(json["position"].is_null());
    assert!(json["queue"].is_null());
}

// Spec: published-api, "Source and device".

#[tokio::test]
async fn qobuz_through_music_assistant() {
    let mut content = device("Example Song", 42, 1_000_000).content.unwrap();
    content.service = Some(Service::new(Brand::new("qobuz"), "Qobuz"));
    let provider =
        FakeProvider::claiming("Music Assistant", Some(content)).with_brand("music-assistant");
    let monitor = PlayerMonitor::new(vec![Arc::new(provider)]);
    let mut player = device("Example Song", 42, 1_000_000);
    player.origin = Origin::Stream {
        url: "http://192.168.1.50:8097/flow/session1/aabbccddeeff/1.flac".into(),
    };
    player.maker = Brand::new("bluesound");
    monitor.apply(&player);

    let json = player_json(&app(&monitor)).await;
    assert_eq!(
        json["source"],
        serde_json::json!({
            "brand": "qobuz",
            "name": "Qobuz",
            "logoUrl": null,
            "via": {
                "brand": "music-assistant",
                "name": "Music Assistant",
                "available": true,
                "logoUrl": null
            }
        })
    );
    assert_eq!(
        json["device"],
        serde_json::json!({"brand": "bluesound", "name": "Example Player", "logoUrl": null, "markUrl": null})
    );
}

// Spec: published-api, "Brand logos".

/// A Bluesound player playing Qobuz itself, Hi-Res.
fn qobuz_on_bluesound() -> DeviceSnapshot {
    let mut player = device("Example Song", 42, 1_000_000);
    player.origin = Origin::Service {
        id: "Qobuz".into(),
        name: "Qobuz".into(),
        cast: false,
        brand: Brand::new("qobuz"),
    };
    player.maker = Brand::new("bluesound");
    player
}

#[tokio::test]
async fn logo_supplied() {
    let logos = Arc::new(TestLogos::default());
    logos.supply("qobuz", 7, "image/svg+xml", b"<svg/>");
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&qobuz_on_bluesound());
    let app = app_with_logos(&monitor, logos);

    let json = player_json(&app).await;
    assert_eq!(json["source"]["logoUrl"], "/api/v1/logos/qobuz?v=7");
    let (status, body, content_type) = get(&app, "/api/v1/logos/qobuz?v=7").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type.as_deref(), Some("image/svg+xml"));
    assert_eq!(body, b"<svg/>");
}

#[tokio::test]
async fn only_the_current_version_may_be_cached() {
    let logos = Arc::new(TestLogos::default());
    logos.supply("qobuz", 8, "image/svg+xml", b"<svg id='new'/>");
    let app = app_with_logos(&PlayerMonitor::new(Vec::new()), logos);
    let cache = |response: &axum::response::Response| {
        response.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .to_owned()
    };

    let current = app
        .clone()
        .oneshot(
            Request::get("/api/v1/logos/qobuz?v=8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(cache(&current).contains("immutable"));

    // An old version's URL gets the current bytes, but not for good.
    let old = app
        .oneshot(
            Request::get("/api/v1/logos/qobuz?v=7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cache(&old), "no-cache");
}

#[tokio::test]
async fn no_logo() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&qobuz_on_bluesound());
    let json = player_json(&app(&monitor)).await;
    assert_eq!(json["source"].get("logoUrl"), Some(&Value::Null));
    assert_eq!(json["device"].get("logoUrl"), Some(&Value::Null));
    assert_eq!(json["quality"].get("hiResLogoUrl"), Some(&Value::Null));
}

#[tokio::test]
async fn logo_added_later() {
    let logos = Arc::new(TestLogos::default());
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&qobuz_on_bluesound());
    let app = app_with_logos(&monitor, Arc::clone(&logos));
    assert_eq!(player_json(&app).await["device"]["logoUrl"], Value::Null);

    logos.supply("bluesound", 3, "image/png", &[0x89, b'P', b'N', b'G']);
    assert_eq!(
        player_json(&app).await["device"]["logoUrl"],
        "/api/v1/logos/bluesound?v=3"
    );
}

#[tokio::test]
async fn hi_res_logo() {
    let logos = Arc::new(TestLogos::default());
    logos.supply("hi-res", 1, "image/png", &[0x89, b'P', b'N', b'G']);
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&qobuz_on_bluesound());
    let json = player_json(&app_with_logos(&monitor, logos)).await;
    assert_eq!(json["quality"]["hiResLogoUrl"], "/api/v1/logos/hi-res?v=1");
}

#[tokio::test]
async fn unknown_brand_requested() {
    let monitor = PlayerMonitor::new(Vec::new());
    let app = app(&monitor);
    for path in [
        "/api/v1/logos/example",
        "/api/v1/logos/..%2F..%2Fetc%2Fpasswd",
        "/api/v1/logos/Qobuz",
    ] {
        let (status, _, _) = get(&app, path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
}

// Spec: published-api, "Quality details".

#[tokio::test]
async fn lossless() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));
    let quality = &player_json(&app(&monitor)).await["quality"];
    assert_eq!(quality["codec"], "FLAC");
    assert_eq!(quality["bitDepth"], 24);
    assert_eq!(quality["sampleRateHz"], 96_000);
    assert_eq!(quality.get("bitrate"), Some(&Value::Null));
}

#[tokio::test]
async fn codec_in_capitals() {
    // Music Assistant reports `flac`, BluOS `FLAC`: the API writes it as the label does.
    let monitor = PlayerMonitor::new(Vec::new());
    let mut lower = device("Example Song", 42, 1_000_000);
    if let Some(content) = lower.content.as_mut() {
        content.quality = Some(AudioQuality::format(AudioFormat {
            codec: "flac".into(),
            bit_depth: 24,
            sample_rate_hz: 88_200,
        }));
    }
    monitor.apply(&lower);
    assert_eq!(
        player_json(&app(&monitor)).await["quality"]["codec"],
        "FLAC"
    );
}

#[tokio::test]
async fn lossy() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut lossy = device("Example Song", 42, 1_000_000);
    if let Some(content) = lossy.content.as_mut() {
        content.quality = Some(AudioQuality::bitrate(160_000));
    }
    monitor.apply(&lossy);
    let quality = &player_json(&app(&monitor)).await["quality"];
    assert_eq!(quality["bitrate"], 160_000);
    for field in ["codec", "bitDepth", "sampleRateHz"] {
        assert_eq!(quality.get(field), Some(&Value::Null), "{field}");
    }
}

// Spec: published-api, "Signal".

fn flac(bit_depth: u32, sample_rate_hz: u32) -> AudioQuality {
    AudioQuality::format(AudioFormat {
        codec: "flac".into(),
        bit_depth,
        sample_rate_hz,
    })
}

/// Qobuz through Music Assistant, delivering `source` as `delivery`.
async fn signal_through_music_assistant(
    source: AudioQuality,
    delivery: Delivery,
    logos: Arc<TestLogos>,
) -> Value {
    let mut content = device("Example Song", 42, 1_000_000).content.unwrap();
    content.service = Some(Service::new(Brand::new("qobuz"), "Qobuz"));
    content.quality = Some(source);
    content.delivery = Some(delivery);
    let provider =
        FakeProvider::claiming("Music Assistant", Some(content)).with_brand("music-assistant");
    let monitor = PlayerMonitor::new(vec![Arc::new(provider)]);
    let mut player = device("Example Song", 42, 1_000_000);
    player.origin = Origin::Stream {
        url: "http://192.168.1.50:8097/flow/session1/aabbccddeeff/1.flac".into(),
    };
    monitor.apply(&player);
    player_json(&app_with_logos(&monitor, logos)).await["signal"].clone()
}

#[tokio::test]
async fn hi_res_through_music_assistant() {
    let logos = Arc::new(TestLogos::default());
    logos.supply("hi-res", 1, "image/png", &[0x89, b'P', b'N', b'G']);
    let signal = signal_through_music_assistant(
        flac(24, 96_000),
        Delivery {
            quality: Some(flac(24, 96_000)),
            changes: vec![Change::Normalised {
                gain: Some(Decibels::from_db(-6.0)),
            }],
        },
        logos,
    )
    .await;

    assert_eq!(signal["class"], "hi-res");
    assert_eq!(signal["source"]["sampleRateHz"], 96_000);
    assert_eq!(signal["delivered"]["sampleRateHz"], 96_000);
    assert_eq!(signal["delivered"]["codec"], "FLAC");
    assert_eq!(
        signal["changes"],
        serde_json::json!([{"kind": "normalised", "gainDb": -6.0}])
    );
    assert_eq!(signal["hiResLogoUrl"], "/api/v1/logos/hi-res?v=1");
}

#[tokio::test]
async fn upsampling_stays_lossless() {
    let logos = Arc::new(TestLogos::default());
    logos.supply("hi-res", 1, "image/png", &[0x89, b'P', b'N', b'G']);
    let signal = signal_through_music_assistant(
        flac(16, 44_100),
        Delivery {
            quality: Some(flac(24, 96_000)),
            changes: vec![Change::Resampled, Change::Dsp],
        },
        logos,
    )
    .await;

    assert_eq!(signal["class"], "lossless");
    assert_eq!(signal["hiResLogoUrl"], Value::Null);
    assert_eq!(
        signal["changes"],
        serde_json::json!([{"kind": "resampled"}, {"kind": "dsp"}])
    );
}

#[tokio::test]
async fn downsampled_hi_res() {
    let signal = signal_through_music_assistant(
        flac(24, 192_000),
        Delivery {
            quality: Some(flac(16, 48_000)),
            changes: vec![Change::Resampled],
        },
        Arc::new(TestLogos::default()),
    )
    .await;
    assert_eq!(signal["class"], "lossless");
}

#[tokio::test]
async fn normalised_without_a_gain() {
    let signal = signal_through_music_assistant(
        flac(24, 96_000),
        Delivery {
            quality: None,
            changes: vec![Change::BitDepth, Change::Normalised { gain: None }],
        },
        Arc::new(TestLogos::default()),
    )
    .await;
    assert_eq!(signal["delivered"], Value::Null);
    assert_eq!(
        signal["changes"],
        serde_json::json!([{"kind": "bitDepth"}, {"kind": "normalised", "gainDb": null}])
    );
}

#[tokio::test]
async fn native_service() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut spotify = device("Example Song", 42, 1_000_000);
    spotify.content.as_mut().unwrap().quality = Some(AudioQuality::bitrate(320_000));
    monitor.apply(&spotify);
    let signal = &player_json(&app(&monitor)).await["signal"];

    assert_eq!(signal["class"], "lossy");
    assert_eq!(signal["source"]["bitrate"], 320_000);
    assert_eq!(signal["delivered"], Value::Null);
    assert_eq!(signal["changes"], serde_json::json!([]));
}

#[tokio::test]
async fn quality_not_reported() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut unknown = device("Example Song", 42, 1_000_000);
    unknown.content.as_mut().unwrap().quality = None;
    monitor.apply(&unknown);
    let signal = &player_json(&app(&monitor)).await["signal"];

    assert_eq!(signal["class"], "unknown");
    assert_eq!(signal["source"], Value::Null);
}

#[tokio::test]
async fn no_signal_for_an_input() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&optical_input(Some(1_000_000)));
    let json = player_json(&app(&monitor)).await;
    assert_eq!(json.get("signal"), Some(&Value::Null));
}

// Spec: published-api, "Cover and backdrop URLs".

#[tokio::test]
async fn track_without_a_cover() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut device = device("Example Song", 42, 1_000_000);
    if let Some(content) = device.content.as_mut() {
        content.artwork = None;
    }
    monitor.apply(&device);

    let json = player_json(&app(&monitor)).await;
    assert_eq!(json.get("artUrl"), Some(&Value::Null));
    assert_eq!(json.get("backdropUrl"), Some(&Value::Null));
}

#[tokio::test]
async fn field_names_and_shapes() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));

    let json = player_json(&app(&monitor)).await;
    assert_eq!(json["online"], true);
    assert_eq!(json["deviceName"], "Example Player");
    assert_eq!(json["session"]["label"], "Qobuz");
    assert_eq!(json["session"]["available"], true);
    assert_eq!(json["track"]["artists"][0], "Example Artist");
    assert_eq!(json["track"]["album"], "Example Album");
    assert_eq!(json["artUrl"], "/api/v1/art/known?size=large");
    assert_eq!(json["backdropUrl"], "/api/v1/art/known?size=backdrop");
    assert_eq!(json["position"]["elapsed"], 42.0);
    assert_eq!(json["position"]["duration"], 214.0);
    assert_eq!(json["position"]["observedAt"], 1_000_000);
    assert_eq!(json["quality"]["label"], "FLAC 24/96");
    assert_eq!(json["quality"]["detail"], "Hi-Res");
    assert_eq!(json["quality"]["hiRes"], true);
    assert_eq!(json["queue"][0]["title"], "Next Song");
    assert_eq!(json["queue"][0]["artUrl"], "/api/v1/art/known");
}

#[tokio::test]
async fn album_colours() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut coloured = device("Example Song", 42, 1_000_000);
    coloured.content.as_mut().unwrap().palette = Some(Palette {
        background: Rgb(73, 51, 38),
        accent: Rgb(209, 147, 108),
    });
    monitor.apply(&coloured);

    let json = player_json(&app(&monitor)).await;
    assert_eq!(
        json["palette"]["background"],
        serde_json::json!([73, 51, 38])
    );
    assert_eq!(
        json["palette"]["accent"],
        serde_json::json!([209, 147, 108])
    );
}

#[tokio::test]
async fn no_album_colours() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));

    let json = player_json(&app(&monitor)).await;
    assert!(json.as_object().unwrap().contains_key("palette"));
    assert!(json["palette"].is_null());
}

fn optical_input(started_at: Option<u64>) -> DeviceSnapshot {
    DeviceSnapshot {
        origin: Origin::Input(AudioInput {
            kind: InputKind::Optical,
            name: "Example Input".into(),
            started_at: started_at.map(Timestamp::from_millis),
        }),
        content: None,
        ..device("unused", 0, 0)
    }
}

#[tokio::test]
async fn input_plays() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&optical_input(Some(1_000_000)));

    let json = player_json(&app(&monitor)).await;
    assert_eq!(json["input"]["kind"], "optical");
    assert_eq!(json["input"]["name"], "Example Input");
    assert_eq!(json["input"]["startedAt"], 1_000_000);
    assert_eq!(json["session"]["label"], "Optical input");
    assert!(json["track"].is_null());
    assert!(json["queue"].is_null());
}

#[tokio::test]
async fn input_waiting_for_a_signal() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&optical_input(None));

    let json = player_json(&app(&monitor)).await;
    assert_eq!(json["input"]["name"], "Example Input");
    assert!(json["input"].as_object().unwrap().contains_key("startedAt"));
    assert!(json["input"]["startedAt"].is_null());
}

#[tokio::test]
async fn input_kinds() {
    for (kind, name) in [
        (InputKind::Analog, "analog"),
        (InputKind::HdmiArc, "hdmi-arc"),
        (InputKind::Other, "other"),
    ] {
        let monitor = PlayerMonitor::new(Vec::new());
        let mut device = optical_input(Some(1));
        if let Origin::Input(input) = &mut device.origin {
            input.kind = kind;
        }
        monitor.apply(&device);
        assert_eq!(player_json(&app(&monitor)).await["input"]["kind"], name);
    }
}

#[tokio::test]
async fn no_input() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));

    let json = player_json(&app(&monitor)).await;
    assert!(json.as_object().unwrap().contains_key("input"));
    assert!(json["input"].is_null());
}

// 7.2 Live updates

/// One SSE event: its name (`None` for unnamed events) and its data.
type SseEvent = (Option<String>, String);

/// Reads SSE events from a response body until `deadline`, returning them
/// and the number of comment lines.
async fn read_sse(body: &mut Body, deadline: Duration) -> (Vec<SseEvent>, usize) {
    let mut events = Vec::new();
    let mut comments = 0;
    let mut name = None;
    let _ = tokio::time::timeout(deadline, async {
        while let Some(Ok(frame)) = body.frame().await {
            let Some(chunk) = frame.data_ref() else {
                continue;
            };
            for line in String::from_utf8_lossy(chunk).lines() {
                if let Some(event) = line.strip_prefix("event:") {
                    name = Some(event.trim().to_owned());
                } else if let Some(payload) = line.strip_prefix("data:") {
                    events.push((name.take(), payload.trim().to_owned()));
                } else if line.starts_with(':') {
                    comments += 1;
                }
            }
        }
    })
    .await;
    (events, comments)
}

/// The snapshots (unnamed events) until `deadline`, and the number of
/// comment lines.
async fn read_events(body: &mut Body, deadline: Duration) -> (Vec<Value>, usize) {
    let (events, comments) = read_sse(body, deadline).await;
    let snapshots = events
        .into_iter()
        .filter(|(name, _)| name.is_none())
        .map(|(_, data)| serde_json::from_str(&data).unwrap())
        .collect();
    (snapshots, comments)
}

async fn open_events(app: &Router) -> Body {
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/player/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    response.into_body()
}

#[tokio::test]
async fn first_event_is_the_current_snapshot() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));
    let mut body = open_events(&app(&monitor)).await;

    let (events, _) = read_events(&mut body, Duration::from_millis(50)).await;
    assert_eq!(events[0]["track"]["title"], "Example Song");
}

#[tokio::test]
async fn track_change_produces_an_event_and_progress_does_not() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Song A", 10, 0));
    let app = app(&monitor);
    let mut body = open_events(&app).await;
    let (first, _) = read_events(&mut body, Duration::from_millis(50)).await;
    assert_eq!(first.len(), 1);

    // Normal progress: no event.
    monitor.apply(&device("Song A", 40, 30_000));
    let (progress, _) = read_events(&mut body, Duration::from_millis(50)).await;
    assert_eq!(progress, Vec::<Value>::new(), "unexpected events");

    // Track change: one event with the new snapshot.
    monitor.apply(&device("Song B", 0, 31_000));
    let (change, _) = read_events(&mut body, Duration::from_millis(50)).await;
    assert_eq!(change.len(), 1);
    assert_eq!(change[0]["track"]["title"], "Song B");
}

#[tokio::test]
async fn heartbeat_comments() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));
    let mut body = open_events(&app(&monitor)).await;

    let (_, comments) = read_events(&mut body, Duration::from_millis(350)).await;
    assert!(
        comments >= 2,
        "expected keep-alive comments, got {comments}"
    );
}

#[test]
fn default_keep_alive_is_within_30_seconds() {
    assert!(HttpConfig::default().keep_alive <= Duration::from_secs(30));
}

// 7.3 No vendor data in the API

#[tokio::test]
async fn responses_contain_no_vendor_data() {
    let monitor = PlayerMonitor::new(Vec::new());
    let mut snapshot = device("Example Song", 42, 1_000_000);
    snapshot.origin = Origin::Stream {
        url: "http://192.168.1.50:8097/flow/x.flac".into(),
    };
    monitor.apply(&snapshot);
    let app = app(&monitor);

    let (_, body, _) = get(&app, "/api/v1/player").await;
    let text = String::from_utf8(body).unwrap();
    for forbidden in [
        "192.168.1.40",
        "192.168.1.50",
        "title1",
        "streamUrl",
        "Artwork?songid",
        "aabbccddeeff",
    ] {
        assert!(
            !text.contains(forbidden),
            "response contains {forbidden:?}: {text}"
        );
    }
}

// 7.4 Artwork

#[tokio::test]
async fn artwork_for_an_issued_id() {
    let monitor = PlayerMonitor::new(Vec::new());
    let (status, body, content_type) = get(&app(&monitor), "/api/v1/art/known").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type.as_deref(), Some("image/jpeg"));
    assert_eq!(body, vec![0xFF, 0xD8]);
}

#[tokio::test]
async fn artwork_sizes_reach_the_source() {
    let monitor = PlayerMonitor::new(Vec::new());
    let app = app(&monitor);
    for (query, bytes) in [
        ("", vec![0xFF, 0xD8]),
        ("?size=large", vec![0xFF, 0xD8, 0x01]),
        ("?size=backdrop", vec![0xFF, 0xD8, 0x02]),
    ] {
        let (status, body, _) = get(&app, &format!("/api/v1/art/known{query}")).await;
        assert_eq!(status, StatusCode::OK, "{query}");
        assert_eq!(body, bytes, "{query}");
    }
    let (status, _, _) = get(&app, "/api/v1/art/known?size=huge").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn artwork_for_an_unknown_id() {
    let monitor = PlayerMonitor::new(Vec::new());
    let (status, _, _) = get(&app(&monitor), "/api/v1/art/0000000000000000").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn artwork_source_fails() {
    let monitor = PlayerMonitor::new(Vec::new());
    let (status, _, _) = get(&app(&monitor), "/api/v1/art/broken").await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
}

// 7.5 Kiosk files

#[tokio::test]
async fn kiosk_at_the_root() {
    let monitor = PlayerMonitor::new(Vec::new());
    let (status, body, content_type) = get(&app(&monitor), "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.unwrap().starts_with("text/html"));
    assert!(String::from_utf8(body).unwrap().contains("<html"));
}

// Spec: published-api, "Kiosk build".

/// The build in the page's `<meta name="jukelight-build">`.
fn page_build(page: &str) -> String {
    let start = page
        .find(r#"<meta name="jukelight-build" content=""#)
        .expect("the page carries its build")
        + r#"<meta name="jukelight-build" content=""#.len();
    page[start..start + page[start..].find('"').unwrap()].to_owned()
}

#[tokio::test]
async fn page_carries_the_build() {
    let monitor = PlayerMonitor::new(Vec::new());
    let (_, body, _) = get(&app(&monitor), "/").await;
    let build = page_build(&String::from_utf8(body).unwrap());
    assert_eq!(build.len(), 12, "a short identifier: {build}");
    assert!(build.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn stream_announces_the_build() {
    let monitor = PlayerMonitor::new(Vec::new());
    let app = app(&monitor);
    let (_, page, _) = get(&app, "/").await;
    let mut body = open_events(&app).await;
    let (events, _) = read_sse(&mut body, Duration::from_millis(50)).await;
    assert_eq!(events.len(), 2, "{events:?}");
    assert_eq!(events[0].0, None, "the snapshot first");
    assert_eq!(
        events[1],
        (
            Some("build".to_owned()),
            page_build(&String::from_utf8(page).unwrap())
        )
    );
}

#[tokio::test]
async fn unknown_pages_fall_back_to_the_kiosk() {
    let monitor = PlayerMonitor::new(Vec::new());
    let (status, body, _) = get(&app(&monitor), "/some/page").await;
    assert_eq!(status, StatusCode::OK);
    assert!(String::from_utf8(body).unwrap().contains("<html"));
}

#[tokio::test]
async fn unknown_api_paths_are_not_found() {
    let monitor = PlayerMonitor::new(Vec::new());
    let (status, _, _) = get(&app(&monitor), "/api/v1/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// Shutdown: live streams end, so a graceful shutdown does not wait forever.

#[tokio::test]
async fn shutdown_ends_live_streams() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));
    let (stop, shutdown) = tokio::sync::watch::channel(false);
    let config = HttpConfig {
        keep_alive: Duration::from_millis(100),
    };
    let app = router_with_shutdown(
        monitor.handle(),
        display(),
        Arc::new(TestArtwork),
        Arc::new(TestLogos::default()),
        &config,
        shutdown,
    );
    let mut body = open_events(&app).await;
    let _ = read_events(&mut body, Duration::from_millis(50)).await;

    stop.send(true).unwrap();
    let ended = tokio::time::timeout(Duration::from_secs(1), async {
        while let Some(frame) = body.frame().await {
            frame.unwrap();
        }
    })
    .await;
    assert!(ended.is_ok(), "the event stream should end on shutdown");
}

#[tokio::test]
async fn streams_without_a_shutdown_signal_keep_running() {
    let monitor = PlayerMonitor::new(Vec::new());
    monitor.apply(&device("Example Song", 42, 1_000_000));
    let mut body = open_events(&app(&monitor)).await;

    // Keep-alive comments keep arriving: the stream is still open.
    let (_, comments) = read_events(&mut body, Duration::from_millis(350)).await;
    assert!(comments >= 2);
}

// Spec: published-api, "Speakers and outputs" and "The maker's mark".
mod speakers_and_outputs {
    use super::*;
    use jukelight_application::SpeakerNames;
    use jukelight_domain::{Output, OutputKind, Speaker, SpeakerKind};

    /// The router, with the monitor it reads from.
    fn group_snapshot_app(logos: Arc<TestLogos>) -> (Router, PlayerMonitor) {
        let mut content = device("Example Song", 42, 1_000_000).content.unwrap();
        content.speakers = vec![
            Speaker {
                name: "Kitchen".into(),
                kind: SpeakerKind::Speaker,
                playing: true,
                quality: Some(AudioQuality::format(AudioFormat {
                    codec: "s16le".into(),
                    bit_depth: 16,
                    sample_rate_hz: 44_100,
                })),
            },
            Speaker {
                name: "Office".into(),
                kind: SpeakerKind::Computer,
                playing: false,
                quality: None,
            },
        ];
        let provider =
            FakeProvider::claiming("Music Assistant", Some(content)).with_brand("music-assistant");
        let monitor = PlayerMonitor::with_names(
            vec![Arc::new(provider)],
            SpeakerNames::new(
                std::collections::HashMap::new(),
                Some("Example Speaker".into()),
            ),
        );
        let mut player = device("Example Song", 42, 1_000_000);
        player.origin = Origin::Stream {
            url: "http://192.168.1.50:8097/flow/session1/aabbccddeeff/1.flac".into(),
        };
        player.maker = Brand::new("bluesound");
        player.outputs = vec![Output {
            name: None,
            kind: OutputKind::Subwoofer,
        }];
        monitor.apply(&player);
        (app_with_logos(&monitor, logos), monitor)
    }

    #[tokio::test]
    async fn recorded_group() {
        let (app, _monitor) = group_snapshot_app(Arc::new(TestLogos::default()));
        let json = player_json(&app).await;
        let speakers = &json["speakers"];
        assert_eq!(speakers[0]["name"], "Kitchen");
        assert_eq!(speakers[0]["kind"], "speaker");
        assert_eq!(speakers[0]["playing"], true);
        assert_eq!(speakers[0]["quality"]["bitDepth"], 16);
        assert_eq!(speakers[0]["quality"]["codec"], "PCM");
        assert_eq!(speakers[0]["quality"]["class"], "lossless");
        assert_eq!(speakers[1]["kind"], "computer");
        assert_eq!(speakers[1]["playing"], false);
        assert_eq!(speakers[1]["quality"], Value::Null);
        assert_eq!(
            json["outputs"],
            serde_json::json!([
                {"name": "Example Speaker", "kind": "speaker"},
                {"name": "Subwoofer", "kind": "subwoofer"}
            ])
        );
    }

    #[tokio::test]
    async fn padded_cd_quality_is_not_hi_res() {
        let flac = |bit_depth, sample_rate_hz| {
            AudioQuality::format(AudioFormat {
                codec: "flac".into(),
                bit_depth,
                sample_rate_hz,
            })
        };
        let mut content = device("Example Song", 42, 1_000_000).content.unwrap();
        content.quality = Some(flac(16, 44_100));
        content.speakers = vec![Speaker {
            name: "Office".into(),
            kind: SpeakerKind::Computer,
            playing: true,
            quality: Some(flac(24, 44_100)),
        }];
        let provider = FakeProvider::claiming("Music Assistant", Some(content));
        let monitor = PlayerMonitor::new(vec![Arc::new(provider)]);
        let mut player = device("Example Song", 42, 1_000_000);
        player.origin = Origin::Stream {
            url: "http://192.168.1.50:8097/flow/session1/aabbccddeeff/1.flac".into(),
        };
        monitor.apply(&player);
        let json = player_json(&app(&monitor)).await;
        assert_eq!(json["speakers"][0]["quality"]["class"], "lossless");
    }

    #[tokio::test]
    async fn playing_alone() {
        let monitor = PlayerMonitor::new(Vec::new());
        monitor.apply(&device("Example Song", 42, 1_000_000));
        let json = player_json(&app(&monitor)).await;
        assert_eq!(json["speakers"], serde_json::json!([]));
        assert_eq!(json["outputs"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn mark_supplied() {
        let logos = Arc::new(TestLogos::default());
        logos.supply("bluesound-mark", 4, "image/svg+xml", b"<svg/>");
        let (app, _monitor) = group_snapshot_app(logos);
        let json = player_json(&app).await;
        assert_eq!(
            json["device"]["markUrl"],
            "/api/v1/logos/bluesound-mark?v=4"
        );
    }

    #[tokio::test]
    async fn no_mark() {
        let (app, _monitor) = group_snapshot_app(Arc::new(TestLogos::default()));
        let json = player_json(&app).await;
        assert_eq!(json["device"]["markUrl"], Value::Null);
    }
}
