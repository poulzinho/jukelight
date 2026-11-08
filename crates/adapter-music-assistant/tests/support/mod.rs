//! A fake Music Assistant for the provider tests: replays recorded
//! responses and records the requests it gets.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared by several test binaries

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use jukelight_adapter_music_assistant::{
    MusicAssistantConfig, MusicAssistantProvider, Timing, Token,
};
use jukelight_application::SessionProvider;
use jukelight_application::testing::FixedClock;
use jukelight_domain::{Content, DeviceSnapshot, Origin, PlaybackState, Timestamp};
use serde_json::Value;

pub const TOKEN: &str = "test-token-0123456789";
pub const WAIT: Duration = Duration::from_secs(5);

/// Fast polling for tests; "Claimed" uses the real interval.
pub const FAST: Timing = Timing {
    interval: Duration::from_millis(50),
    request_timeout: Duration::from_millis(300),
};

/// A fake Music Assistant serving one recorded situation.
pub struct Server {
    fixture: String,
    /// Replaces fields of the recorded queue.
    queue_patch: Option<Value>,
    reachable: bool,
    requests: Vec<(String, Value)>,
}

#[derive(Clone)]
pub struct TestServer {
    state: Arc<Mutex<Server>>,
    pub url: String,
}

impl TestServer {
    pub async fn start(fixture: &str) -> Self {
        let state = Arc::new(Mutex::new(Server {
            fixture: fixture.into(),
            queue_patch: None,
            reachable: true,
            requests: Vec::new(),
        }));
        let app = Router::new()
            .route("/api", post(api))
            .with_state(Arc::clone(&state));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            state,
            url: format!("http://127.0.0.1:{port}"),
        }
    }

    pub fn set_fixture(&self, fixture: &str) {
        self.state.lock().unwrap().fixture = fixture.into();
    }

    pub fn patch_queue(&self, patch: Value) {
        self.state.lock().unwrap().queue_patch = Some(patch);
    }

    pub fn set_reachable(&self, reachable: bool) {
        self.state.lock().unwrap().reachable = reachable;
    }

    pub fn requests(&self, command: &str) -> Vec<Value> {
        self.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|(name, _)| name == command)
            .map(|(_, args)| args.clone())
            .collect()
    }

    pub fn request_count(&self) -> usize {
        self.state.lock().unwrap().requests.len()
    }

    pub fn provider(&self, token: &str, timing: Timing) -> Arc<MusicAssistantProvider> {
        let config = MusicAssistantConfig {
            url: self.url.clone(),
            token: Token::new(token),
        };
        let provider = Arc::new(
            MusicAssistantProvider::new(&config, Arc::new(FixedClock::at(now().as_millis())))
                .unwrap()
                .with_timing(timing),
        );
        tokio::spawn(provider.run());
        provider
    }
}

pub async fn api(
    State(state): State<Arc<Mutex<Server>>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let reachable = state.lock().unwrap().reachable;
    if !reachable {
        // Does not answer within the provider's timeout.
        tokio::time::sleep(Duration::from_secs(2)).await;
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if headers.get("authorization").and_then(|v| v.to_str().ok())
        != Some(&format!("Bearer {TOKEN}"))
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let command = body["command"].as_str().unwrap_or_default().to_owned();
    let mut server = state.lock().unwrap();
    server
        .requests
        .push((command.clone(), body["args"].clone()));
    let file = match command.as_str() {
        "players/get" => "player.json",
        "player_queues/get" => "queue.json",
        "player_queues/items" => "items.json",
        "providers" => "providers.json",
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let mut value = fixture(&server.fixture, file);
    if let (Some(patch), "queue.json") = (&server.queue_patch, file) {
        for (key, field) in patch.as_object().unwrap() {
            value[key] = field.clone();
        }
    }
    Json(value).into_response()
}

pub fn fixture(fixture: &str, file: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture)
        .join(file);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// One second after the Qobuz recording measured its position.
pub fn now() -> Timestamp {
    Timestamp::from_millis(1_793_395_218_474)
}

pub fn device(origin: Origin) -> DeviceSnapshot {
    DeviceSnapshot {
        maker: None,
        name: "Living Room".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin,
        content: None,
    }
}

/// The flow stream Music Assistant sends to the test player.
pub fn claimed_device() -> DeviceSnapshot {
    device(Origin::Stream {
        url: "http://127.0.0.1:8097/flow/session1/aabbccddeeff/00000000000000000000000000000001/aabbccddeeff.flac".into(),
    })
}

pub fn service_device() -> DeviceSnapshot {
    device(Origin::Service {
        brand: None,
        id: "Qobuz".into(),
        name: "Qobuz".into(),
        cast: false,
    })
}

/// Polls `provider.content()` until it matches, for at most `WAIT`.
pub async fn wait_for(
    provider: &MusicAssistantProvider,
    matches: impl Fn(&Option<Content>) -> bool,
) -> Option<Content> {
    let deadline = tokio::time::Instant::now() + WAIT;
    loop {
        let content = provider.content();
        if matches(&content) {
            return content;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "content never matched; last: {content:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

pub fn title(content: Option<&Content>) -> Option<&str> {
    content.map(|content| content.track.title.as_str())
}
