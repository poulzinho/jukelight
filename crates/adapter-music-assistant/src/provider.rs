use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::StreamExt;
use futures::stream::{self, BoxStream};
use jukelight_application::{Clock, SessionProvider};
use jukelight_domain::{Brand, Content, DeviceSnapshot, Origin, PlaybackState};
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
    /// Time between the starts of two polls while claimed or checking.
    pub interval: Duration,
    pub request_timeout: Duration,
    /// How long Music Assistant's answer counts for an AirPlay claim.
    pub answer_lifetime: Duration,
    /// How long a paused group is held while Music Assistant still reports
    /// its queue playing: it reports a pause late.
    pub pause_lag: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(2),
            request_timeout: Duration::from_secs(5),
            answer_lifetime: Duration::from_secs(5),
            pause_lag: Duration::from_secs(5),
        }
    }
}

/// Why the provider polls Music Assistant.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    /// Music Assistant's own stream: claimed from configuration alone.
    Claimed,
    /// An AirPlay session: claimed only while Music Assistant says it drives
    /// the player (it plays to a group of speakers).
    Checking,
    /// The player stopped after a group session: claimed while Music
    /// Assistant's queue still holds the group's item, shown as paused.
    Held(Group),
}

/// The last group session the provider claimed: the queue it played from
/// and its current item.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Group {
    queue: String,
    item: String,
}

/// Music Assistant's latest answer about an AirPlay session: the queue it
/// plays to the player from, if any, and when it answered.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Answer {
    queue: Option<String>,
    at: Instant,
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
    /// The player key and why it is followed, else `None`.
    followed: watch::Sender<Option<(String, Mode)>>,
    content: watch::Sender<Option<Content>>,
    /// Music Assistant's latest answer during an AirPlay session; it ticks
    /// `updates` when the queue changes, so the claim is decided again.
    answer: watch::Sender<Option<Answer>>,
    /// When the current hold of a paused group began.
    held_since: std::sync::Mutex<Option<Instant>>,
    /// Whether the last request to Music Assistant was answered: a provider
    /// that can't reach it isn't settling anything.
    reachable: AtomicBool,
    /// The last group session claimed, kept while the player stays stopped
    /// after it, so a paused group can be held.
    group: std::sync::Mutex<Option<Group>>,
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
                answer: watch::Sender::new(None),
                held_since: std::sync::Mutex::new(None),
                reachable: AtomicBool::new(true),
                group: std::sync::Mutex::new(None),
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

async fn run(inner: std::sync::Weak<Inner>, mut followed: watch::Receiver<Option<(String, Mode)>>) {
    let mut cache = Cache::default();
    let mut reachable = true;
    loop {
        let (player_key, mode) = {
            let Ok(key) = followed.wait_for(Option::is_some).await else {
                return;
            };
            key.clone().unwrap_or((String::new(), Mode::Claimed))
        };
        let held = matches!(mode, Mode::Held(_));
        let Some(provider) = inner.upgrade() else {
            return;
        };
        let started = Instant::now();
        match provider.poll(&player_key, mode, &mut cache).await {
            Ok(content) => {
                if !reachable {
                    tracing::info!("Music Assistant reachable again");
                }
                reachable = true;
                provider.reachable.store(true, Ordering::Relaxed);
                if held && content.is_none() {
                    // Music Assistant let go of the group's item: stop following.
                    provider.release();
                }
                provider.publish(content);
            }
            Err(error) => {
                if reachable {
                    tracing::warn!(%error, "Music Assistant unreachable");
                } else {
                    tracing::debug!(%error, "Music Assistant still unreachable");
                }
                reachable = false;
                provider.reachable.store(false, Ordering::Relaxed);
                cache = Cache::default();
                provider.publish(None);
                provider.forget_answer();
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
                provider.forget_answer();
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

    /// Remembers which queue Music Assistant plays to the player from during
    /// an AirPlay session; ticks `updates` only when that changes.
    fn remember_answer(&self, queue: Option<String>) {
        let at = Instant::now();
        self.answer.send_if_modified(|current| {
            let changed = current.as_ref().map(|answer| &answer.queue) != Some(&queue);
            *current = Some(Answer { queue, at });
            changed
        });
    }

    fn group(&self) -> Option<Group> {
        self.group.lock().ok().and_then(|group| group.clone())
    }

    fn remember_group(&self, group: Option<Group>) {
        if let Ok(mut current) = self.group.lock() {
            *current = group;
        }
    }

    /// Ends a hold: forgets the group and stops following the player.
    fn release(&self) {
        self.remember_group(None);
        self.hold_from(None);
        self.followed
            .send_if_modified(|current| current.take().is_some());
    }

    /// Starts (when not started yet) or ends the clock of a hold.
    fn hold_from(&self, start: Option<Instant>) {
        if let Ok(mut since) = self.held_since.lock() {
            match start {
                Some(now) => {
                    since.get_or_insert(now);
                }
                None => *since = None,
            }
        }
    }

    /// Whether the hold has lasted longer than Music Assistant's lag in
    /// reporting a pause.
    fn hold_outlasted_the_lag(&self) -> bool {
        self.held_since
            .lock()
            .ok()
            .and_then(|since| *since)
            .is_some_and(|since| since.elapsed() > self.timing.pause_lag)
    }

    /// Whether the provider holds a paused group session.
    fn holds(&self) -> bool {
        self.content
            .borrow()
            .as_ref()
            .is_some_and(|content| content.held)
    }

    fn forget_answer(&self) {
        self.answer
            .send_if_modified(|current| current.take().is_some());
    }

    /// Whether Music Assistant said, recently enough, that it drives the
    /// player.
    fn drives_player(&self) -> bool {
        self.answer.borrow().as_ref().is_some_and(|answer| {
            answer.queue.is_some() && answer.at.elapsed() <= self.timing.answer_lifetime
        })
    }

    async fn poll(
        &self,
        player_key: &str,
        mode: Mode,
        cache: &mut Cache,
    ) -> Result<Option<Content>, PollError> {
        let player: api::Player = self
            .call("players/get", json!({ "player_id": player_key }))
            .await?;
        let group_queue = mapping::music_assistant_queue(&player, player_key);
        let queue_id = match &mode {
            Mode::Claimed => group_queue.unwrap_or_else(|| player_key.to_owned()),
            Mode::Checking => {
                self.remember_answer(group_queue.clone());
                // A phone's AirPlay: nothing more to ask.
                let Some(queue_id) = group_queue else {
                    return Ok(None);
                };
                queue_id
            }
            Mode::Held(group) => group.queue.clone(),
        };
        let queue: api::Queue = self
            .call("player_queues/get", json!({ "queue_id": queue_id }))
            .await?;
        let item = queue
            .current_item
            .as_ref()
            .map(|item| item.queue_item_id.clone());
        match &mode {
            Mode::Held(group) => {
                // Held while the queue still holds the group's item. Music
                // Assistant reports a pause late, so a queue still playing it
                // is held too, until the lag has passed: then the group plays
                // on without the player.
                let playing = queue.state.as_deref() == Some("playing");
                if item.as_ref() != Some(&group.item) || (playing && self.hold_outlasted_the_lag())
                {
                    return Ok(None);
                }
            }
            Mode::Checking => self.remember_group(item.map(|item| Group {
                queue: queue_id.clone(),
                item,
            })),
            Mode::Claimed => {}
        }

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
                        json!({ "queue_id": queue_id, "limit": UP_NEXT, "offset": index + 1 }),
                    )
                    .await?
                }
                None => Vec::new(),
            };
            cache.queue_key = queue_key;
        }

        // The group's other players, only while in a group.
        let players = if mapping::in_group(&player) {
            self.call("players/all", json!({})).await?
        } else {
            Vec::new()
        };
        let mut responses = Responses {
            player,
            queue,
            upcoming: std::mem::take(&mut cache.upcoming),
            providers: cache.providers.take().unwrap_or_default(),
            players,
        };
        // Provider names are reloaded when an unknown instance appears.
        let unknown = mapping::provider_instance(&responses)
            .is_some_and(|instance| !mapping::knows_provider(instance, &responses.providers));
        if unknown || responses.providers.is_empty() {
            responses.providers = self.call("providers", json!({})).await?;
        }

        let content = mapping::to_content(&responses, &self.base_url, player_key, self.clock.now())
            .map(|content| Content {
                held: matches!(mode, Mode::Held(_)),
                ..content
            });
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
            || (is_airplay(device) && self.inner.drives_player())
            || (device.state == PlaybackState::Stopped && self.inner.holds())
    }

    fn content(&self) -> Option<Content> {
        self.inner.content.borrow().clone()
    }

    /// Settling while Music Assistant hasn't answered for an AirPlay origin
    /// yet, or while the provider claims the origin without content for it;
    /// never while Music Assistant is unreachable.
    fn settling(&self, device: &DeviceSnapshot) -> bool {
        if !self.inner.reachable.load(Ordering::Relaxed) {
            return false;
        }
        let claimed = self.claims(device);
        if claimed {
            return self.inner.content.borrow().is_none();
        }
        is_airplay(device) && self.inner.answer.borrow().is_none()
    }

    fn follow(&self, device: &DeviceSnapshot) {
        let stopped = device.state == PlaybackState::Stopped;
        let mode = if self.inner.rule.claims(device) {
            Some(Mode::Claimed)
        } else if let Some(group) = self.inner.group().filter(|_| stopped) {
            // The group stopped (paused): hold it, showing what was playing
            // as paused until the first poll confirms it.
            self.inner
                .content
                .send_if_modified(|content| match content {
                    Some(content) if !content.held => {
                        content.held = true;
                        true
                    }
                    _ => false,
                });
            self.inner.hold_from(Some(Instant::now()));
            Some(Mode::Held(group))
        } else if is_airplay(device) && !device.player_key.is_empty() {
            self.inner.hold_from(None);
            Some(Mode::Checking)
        } else {
            // Anything else plays: no group to hold any more.
            if !stopped {
                self.inner.remember_group(None);
            }
            None
        };
        let key = mode.map(|mode| (device.player_key.clone(), mode));
        self.inner.followed.send_if_modified(|current| {
            let changed = *current != key;
            *current = key;
            changed
        });
    }

    fn updates(&self) -> BoxStream<'static, ()> {
        let content = stream::unfold(self.inner.content.subscribe(), |mut receiver| async move {
            receiver.changed().await.ok().map(|()| ((), receiver))
        });
        let answer = stream::unfold(self.inner.answer.subscribe(), |mut receiver| async move {
            receiver.changed().await.ok().map(|()| ((), receiver))
        });
        stream::select(content, answer).boxed()
    }
}

/// The player reports Music Assistant's group playback as an AirPlay session,
/// like a phone's AirPlay; only Music Assistant can tell them apart.
fn is_airplay(device: &DeviceSnapshot) -> bool {
    matches!(&device.origin, Origin::Service { id, .. } if id == "AirPlay")
}
