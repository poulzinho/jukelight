use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures::stream::{self, BoxStream};
use jukelight_application::{Clock, DeviceMonitor};
use jukelight_domain::{DeviceSnapshot, Origin, PlaybackState};
use tokio::sync::mpsc;
use tokio::time::Instant;

use crate::parse::{self, Identity, ParseError, PlaylistEntry};
use crate::{is_native, to_device_snapshot};

/// Where the player is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluosConfig {
    pub host: String,
    pub port: u16,
}

/// Timing of requests to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// Longest wait for a change in one long-poll.
    pub long_poll: Duration,
    /// Shortest time between the starts of two requests.
    pub min_spacing: Duration,
    /// First delay after a failed request; it doubles up to `backoff_max`.
    pub backoff_start: Duration,
    pub backoff_max: Duration,
    /// Timeout of requests that are not long-polls.
    pub request_timeout: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            long_poll: Duration::from_secs(100),
            min_spacing: Duration::from_secs(1),
            backoff_start: Duration::from_secs(1),
            backoff_max: Duration::from_secs(30),
            request_timeout: Duration::from_secs(5),
        }
    }
}

/// Follows a BluOS player by long-polling `/Status`.
pub struct BluosMonitor {
    inner: Arc<Inner>,
}

struct Inner {
    base_url: String,
    timing: Timing,
    clock: Arc<dyn Clock>,
    client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
enum PollError {
    #[error("request failed: {0}")]
    Http(reqwest::Error),
    #[error("player answered {0}")]
    Status(reqwest::StatusCode),
    #[error("unreadable response: {0}")]
    Parse(#[from] ParseError),
}

impl From<reqwest::Error> for PollError {
    /// Drops the URL: it contains the player's address, which must not
    /// appear in logs.
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error.without_url())
    }
}

/// What the monitor remembers between polls of one connection.
#[derive(Default)]
struct Session {
    identity: Option<Identity>,
    etag: Option<String>,
    playlist_id: Option<String>,
    playlist: Vec<PlaylistEntry>,
}

impl BluosMonitor {
    pub fn new(config: &BluosConfig, clock: Arc<dyn Clock>) -> Self {
        let base_url = format!("http://{}:{}", config.host, config.port);
        Self {
            inner: Arc::new(Inner {
                base_url,
                timing: Timing::default(),
                clock,
                client: reqwest::Client::new(),
            }),
        }
    }

    #[must_use]
    pub fn with_timing(self, timing: Timing) -> Self {
        let inner = Inner {
            base_url: self.inner.base_url.clone(),
            timing,
            clock: Arc::clone(&self.inner.clock),
            client: self.inner.client.clone(),
        };
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl DeviceMonitor for BluosMonitor {
    fn snapshots(&self) -> BoxStream<'static, DeviceSnapshot> {
        let (sender, receiver) = mpsc::channel(16);
        tokio::spawn(Arc::clone(&self.inner).run(sender));
        stream::unfold(receiver, |mut receiver| async move {
            receiver.recv().await.map(|snapshot| (snapshot, receiver))
        })
        .boxed()
    }
}

impl Inner {
    async fn run(self: Arc<Self>, sender: mpsc::Sender<DeviceSnapshot>) {
        let mut session = Session::default();
        let mut last: Option<DeviceSnapshot> = None;
        let mut backoff = self.timing.backoff_start;

        while !sender.is_closed() {
            let started = Instant::now();
            match self.poll(&mut session).await {
                Ok(snapshot) => {
                    backoff = self.timing.backoff_start;
                    if last.as_ref() != Some(&snapshot)
                        && sender.send(snapshot.clone()).await.is_err()
                    {
                        return;
                    }
                    last = Some(snapshot);
                    tokio::time::sleep_until(started + self.timing.min_spacing).await;
                }
                Err(error) => {
                    tracing::warn!(%error, retry_in = ?backoff, "player unreachable");
                    // Identity and playlist are read again after reconnecting.
                    session = Session::default();
                    let offline = offline(last.as_ref());
                    if last.as_ref().is_none_or(|previous| previous.online) {
                        if sender.send(offline.clone()).await.is_err() {
                            return;
                        }
                        last = Some(offline);
                    }
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(self.timing.backoff_max);
                }
            }
        }
    }

    async fn poll(&self, session: &mut Session) -> Result<DeviceSnapshot, PollError> {
        if session.identity.is_none() {
            let body = self.get("/SyncStatus", self.timing.request_timeout).await?;
            session.identity = Some(parse::parse_sync_status(&body)?);
        }
        let Some(identity) = session.identity.as_ref() else {
            unreachable!("identity set above")
        };

        let (path, timeout) = match &session.etag {
            Some(etag) => (
                format!(
                    "/Status?timeout={}&etag={etag}",
                    self.timing.long_poll.as_secs()
                ),
                self.timing.long_poll + Duration::from_secs(10),
            ),
            None => ("/Status".to_owned(), self.timing.request_timeout),
        };
        let status = parse::parse_status(&self.get(&path, timeout).await?)?;
        let now = self.clock.now();

        // The playlist only matters for native sessions, and only changes
        // with the playlist id.
        let origin = to_device_snapshot(&status, identity, &[], &self.base_url, now).origin;
        let playlist_id = status.get("pid").map(str::to_owned);
        if is_native(&origin) && playlist_id != session.playlist_id {
            let body = self.get("/Playlist", self.timing.request_timeout).await?;
            session.playlist = parse::parse_playlist(&body)?;
            session.playlist_id = playlist_id;
        }

        session.etag = Some(status.etag.clone());
        Ok(to_device_snapshot(
            &status,
            identity,
            &session.playlist,
            &self.base_url,
            now,
        ))
    }

    async fn get(&self, path: &str, timeout: Duration) -> Result<String, PollError> {
        let response = self
            .client
            .get(format!("{}{path}", self.base_url))
            .timeout(timeout)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(PollError::Status(response.status()));
        }
        Ok(response.text().await?)
    }
}

/// The last snapshot marked offline, so the last content stays visible.
fn offline(last: Option<&DeviceSnapshot>) -> DeviceSnapshot {
    match last {
        Some(snapshot) => DeviceSnapshot {
            online: false,
            ..snapshot.clone()
        },
        None => DeviceSnapshot {
            name: String::new(),
            player_key: String::new(),
            online: false,
            state: PlaybackState::Stopped,
            origin: Origin::None,
            content: None,
        },
    }
}
