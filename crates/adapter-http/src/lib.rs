//! HTTP adapter: the published API (v1), live updates and display decisions
//! over Server-Sent Events, the artwork proxy endpoint, and the embedded
//! kiosk.

mod dto;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use futures::{Stream, StreamExt, stream};
use jukelight_application::{ArtworkError, ArtworkId, ArtworkSource, DisplayHandle, PlayerHandle};
use jukelight_domain::DisplayPower;
use rust_embed::RustEmbed;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

pub use dto::PlayerDto;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpConfig {
    /// How often an idle event stream sends a comment line.
    pub keep_alive: Duration,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            keep_alive: Duration::from_secs(15),
        }
    }
}

#[derive(Clone)]
struct AppState {
    player: PlayerHandle,
    display: DisplayHandle,
    artwork: Arc<dyn ArtworkSource>,
    keep_alive: Duration,
    shutdown: watch::Receiver<bool>,
}

/// The kiosk build, embedded at compile time (see build.rs).
#[derive(RustEmbed)]
#[folder = "$OUT_DIR/kiosk"]
struct Kiosk;

/// The router without a shutdown signal: live streams run until clients leave.
pub fn router(
    player: PlayerHandle,
    display: DisplayHandle,
    artwork: Arc<dyn ArtworkSource>,
    config: &HttpConfig,
) -> Router {
    let (_, never) = watch::channel(false);
    router_with_shutdown(player, display, artwork, config, never)
}

/// The router; setting `shutdown` to `true` ends every live event stream, so a
/// graceful shutdown does not wait for kiosks that never disconnect.
pub fn router_with_shutdown(
    player: PlayerHandle,
    display: DisplayHandle,
    artwork: Arc<dyn ArtworkSource>,
    config: &HttpConfig,
    shutdown: watch::Receiver<bool>,
) -> Router {
    let state = AppState {
        player,
        display,
        artwork,
        keep_alive: config.keep_alive,
        shutdown,
    };
    Router::new()
        .route("/api/v1/player", get(player_snapshot))
        .route("/api/v1/player/events", get(player_events))
        .route("/api/v1/display", get(display_events))
        .route("/api/v1/art/{id}", get(artwork_image))
        .route("/api/{*rest}", get(|| async { StatusCode::NOT_FOUND }))
        .fallback(get(kiosk))
        .with_state(state)
}

async fn player_snapshot(State(state): State<AppState>) -> Json<PlayerDto> {
    Json(dto::player_dto(
        &state.player.current(),
        state.artwork.as_ref(),
    ))
}

/// The current snapshot first, then every snapshot that changed beyond
/// normal progress.
async fn player_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let artwork = Arc::clone(&state.artwork);
    let current = stream::once(futures::future::ready(state.player.current()));
    let changes = WatchStream::from_changes(state.player.subscribe_published());
    let events = current.chain(changes).filter_map(move |snapshot| {
        let event = Event::default().json_data(dto::player_dto(&snapshot, artwork.as_ref()));
        futures::future::ready(event.ok().map(Ok))
    });
    let events = events.take_until(shutdown_requested(state.shutdown));
    Sse::new(events).keep_alive(KeepAlive::new().interval(state.keep_alive))
}

/// The current display decision first, then every new one, as `display`
/// events with the data `on` or `off`.
async fn display_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = WatchStream::new(state.display.subscribe())
        .map(|power| {
            let data = match power {
                DisplayPower::On => "on",
                DisplayPower::Off => "off",
            };
            Ok(Event::default().event("display").data(data))
        })
        .take_until(shutdown_requested(state.shutdown));
    Sse::new(events).keep_alive(KeepAlive::new().interval(state.keep_alive))
}

/// Resolves once shutdown is requested; never, if no one can request it.
async fn shutdown_requested(mut shutdown: watch::Receiver<bool>) {
    if shutdown.wait_for(|stop| *stop).await.is_err() {
        std::future::pending::<()>().await;
    }
}

async fn artwork_image(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match state.artwork.fetch(&ArtworkId::new(id)).await {
        // Ids are derived from the source, so an id's image never changes.
        Ok(image) => (
            [
                (header::CONTENT_TYPE, image.content_type),
                (header::CACHE_CONTROL, "public, max-age=86400".into()),
            ],
            image.bytes,
        )
            .into_response(),
        Err(ArtworkError::Unknown) => StatusCode::NOT_FOUND.into_response(),
        Err(ArtworkError::Source(error)) => {
            tracing::debug!(%error, "artwork source failed");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

/// Kiosk files; any other path gets `index.html`, so client-side routes work.
async fn kiosk(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let (file, path) = match Kiosk::get(path).filter(|_| !path.is_empty()) {
        Some(file) => (file, path),
        None => match Kiosk::get("index.html") {
            Some(file) => (file, "index.html"),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };
    // Built assets carry a content hash in their name; the page itself must
    // always be fetched fresh.
    let cache = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [
            (header::CONTENT_TYPE, file.metadata.mimetype().to_owned()),
            (header::CACHE_CONTROL, cache.to_owned()),
        ],
        Body::from(file.data.into_owned()),
    )
        .into_response()
}
