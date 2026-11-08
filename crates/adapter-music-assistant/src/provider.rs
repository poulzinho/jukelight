use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures::stream::{self, BoxStream};
use jukelight_application::{Clock, SessionProvider};
use jukelight_domain::{Brand, Content, DeviceSnapshot};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio::sync::watch;
use tokio::time::Instant;

use crate::api;
use crate::claim::{ClaimRule, UrlError, http_url};
use crate::mapping::{self, Responses, UP_NEXT};

const LABEL: &str = "Music Assistant";

/// A Music Assistant token. Never printed.
#[derive(Clone, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

/// Where Music Assistant is and how to authenticate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicAssistantConfig {
    /// Base URL of Music Assistant, e.g. `http://192.168.1.50:8095`.
    pub url: String,
    pub token: Token,
}

/// Timing of requests to Music Assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// Time between the starts of two polls while claimed.
    pub interval: Duration,
    pub request_timeout: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(2),
            request_timeout: Duration::from_secs(5),
        }
    }
}

/// Follows Music Assistant's view of the player while Music Assistant drives
/// it, by polling its HTTP API. `run()` does the polling and must be spawned.
pub struct MusicAssistantProvider {
    inner: Arc<Inner>,
}

struct Inner {
    rule: ClaimRule,
    base_url: String,
    token: Token,
    timing: Timing,
    clock: Arc<dyn Clock>,
    client: reqwest::Client,
    /// The player key while the provider claims the device, else `None`.
    followed: watch::Sender<Option<String>>,
    content: watch::Sender<Option<Content>>,
}

#[derive(Debug, thiserror::Error)]
enum PollError {
    #[error("request failed: {0}")]
    Http(reqwest::Error),
    #[error("Music Assistant answered {0}")]
    Status(reqwest::StatusCode),
}

impl From<reqwest::Error> for PollError {
    /// Drops the URL: it contains Music Assistant's address, which must not
    /// appear in logs.
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error.without_url())
    }
}

/// What the provider remembers between polls.
#[derive(Default)]
struct Cache {
    /// Current item, index and item count the upcoming entries belong to.
    queue_key: Option<(String, Option<u32>, u32)>,
    upcoming: Vec<api::QueueItem>,
    providers: Option<Vec<api::ProviderInfo>>,
}

impl MusicAssistantProvider {
    pub fn new(config: &MusicAssistantConfig, clock: Arc<dyn Clock>) -> Result<Self, UrlError> {
        let rule = ClaimRule::new(&config.url)?;
        let base_url = http_url(&config.url)?
            .as_str()
            .trim_end_matches('/')
            .to_owned();
        Ok(Self {
            inner: Arc::new(Inner {
                rule,
                base_url,
                token: config.token.clone(),
                timing: Timing::default(),
                clock,
                client: reqwest::Client::new(),
                followed: watch::Sender::new(None),
                content: watch::Sender::new(None),
            }),
        })
    }

    /// Replaces the timing; only before `run()` is called.
    #[must_use]
    pub fn with_timing(mut self, timing: Timing) -> Self {
        if let Some(inner) = Arc::get_mut(&mut self.inner) {
            inner.timing = timing;
        }
        self
    }

    /// Polls Music Assistant while the provider claims the device. Runs
    /// until the provider is dropped.
    pub fn run(&self) -> impl Future<Output = ()> + Send + 'static {
        let inner = Arc::downgrade(&self.inner);
        let followed = self.inner.followed.subscribe();
        run(inner, followed)
    }
}

async fn run(inner: std::sync::Weak<Inner>, mut followed: watch::Receiver<Option<String>>) {
    let mut cache = Cache::default();
    let mut reachable = true;
    loop {
        let player_key = {
            let Ok(key) = followed.wait_for(Option::is_some).await else {
                return;
            };
            key.clone().unwrap_or_default()
        };
        let Some(provider) = inner.upgrade() else {
            return;
        };
        let started = Instant::now();
        match provider.poll(&player_key, &mut cache).await {
            Ok(content) => {
                if !reachable {
                    tracing::info!("Music Assistant reachable again");
                }
                reachable = true;
                provider.publish(content);
            }
            Err(error) => {
                if reachable {
                    tracing::warn!(%error, "Music Assistant unreachable");
                } else {
                    tracing::debug!(%error, "Music Assistant still unreachable");
                }
                reachable = false;
                cache = Cache::default();
                provider.publish(None);
            }
        }
        let interval = provider.timing.interval;
        drop(provider);

        // Wait for the next poll, or stop early when the claim changes.
        tokio::select! {
            () = tokio::time::sleep_until(started + interval) => {}
            changed = followed.changed() => if changed.is_err() { return },
        }
        if followed.borrow().is_none() {
            if let Some(provider) = inner.upgrade() {
                provider.publish(None);
            }
            cache = Cache::default();
        }
    }
}

impl Inner {
    fn publish(&self, content: Option<Content>) {
        self.content.send_if_modified(|current| {
            let changed = *current != content;
            *current = content;
            changed
        });
    }

    async fn poll(
        &self,
        player_key: &str,
        cache: &mut Cache,
    ) -> Result<Option<Content>, PollError> {
        let player: api::Player = self
            .call("players/get", json!({ "player_id": player_key }))
            .await?;
        let queue: api::Queue = self
            .call("player_queues/get", json!({ "queue_id": player_key }))
            .await?;

        // Upcoming entries only change with the current item, index or count.
        let queue_key = queue
            .current_item
            .as_ref()
            .map(|item| (item.queue_item_id.clone(), queue.current_index, queue.items));
        if queue_key != cache.queue_key {
            cache.upcoming = match queue.current_index {
                Some(index) => {
                    self.call(
                        "player_queues/items",
                        json!({ "queue_id": player_key, "limit": UP_NEXT, "offset": index + 1 }),
                    )
                    .await?
                }
                None => Vec::new(),
            };
            cache.queue_key = queue_key;
        }

        let mut responses = Responses {
            player,
            queue,
            upcoming: std::mem::take(&mut cache.upcoming),
            providers: cache.providers.take().unwrap_or_default(),
        };
        // Provider names are reloaded when an unknown instance appears.
        let unknown = mapping::provider_instance(&responses)
            .is_some_and(|instance| !mapping::knows_provider(instance, &responses.providers));
        if unknown || responses.providers.is_empty() {
            responses.providers = self.call("providers", json!({})).await?;
        }

        let content = mapping::to_content(&responses, &self.base_url, player_key, self.clock.now());
        cache.upcoming = responses.upcoming;
        cache.providers = Some(responses.providers);
        Ok(content)
    }

    async fn call<T: DeserializeOwned>(&self, command: &str, args: Value) -> Result<T, PollError> {
        let response = self
            .client
            .post(format!("{}/api", self.base_url))
            .bearer_auth(&self.token.0)
            .timeout(self.timing.request_timeout)
            .json(&json!({ "command": command, "args": args }))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(PollError::Status(response.status()));
        }
        Ok(response.json().await?)
    }
}

impl SessionProvider for MusicAssistantProvider {
    fn label(&self) -> &str {
        LABEL
    }

    fn brand(&self) -> Option<Brand> {
        Brand::new("music-assistant")
    }

    fn claims(&self, device: &DeviceSnapshot) -> bool {
        self.inner.rule.claims(device)
    }

    fn content(&self) -> Option<Content> {
        self.inner.content.borrow().clone()
    }

    fn follow(&self, device: &DeviceSnapshot) {
        let key = self.claims(device).then(|| device.player_key.clone());
        self.inner.followed.send_if_modified(|current| {
            let changed = *current != key;
            *current = key;
            changed
        });
    }

    fn updates(&self) -> BoxStream<'static, ()> {
        stream::unfold(self.inner.content.subscribe(), |mut receiver| async move {
            receiver.changed().await.ok().map(|()| ((), receiver))
        })
        .boxed()
    }
}
