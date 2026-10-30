//! A fake device with scripted scenarios, for kiosk work, demos and tests
//! without a real player. Selected with `JUKELIGHT_DEVICE=fake:<scenario>`.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::StreamExt;
use futures::stream::{self, BoxStream};
use jukelight_application::{Clock, DeviceMonitor};
use jukelight_domain::{
    ArtworkRef, AudioFormat, AudioQuality, Content, DeviceSnapshot, Origin, PlaybackState,
    Position, QueueEntry, QueueView, Track,
};
use tokio::sync::watch;

pub const SCENARIOS: [&str; 8] = [
    "playing-with-queue",
    "cast-without-queue",
    "long-title",
    "no-artwork",
    "hi-res",
    "paused",
    "stopped",
    "offline",
];

#[derive(Debug, thiserror::Error)]
#[error("unknown scenario {0:?}; choose one of: {list}", list = SCENARIOS.join(", "))]
pub struct UnknownScenario(String);

/// A device that reports one scenario, until told it is unreachable.
pub struct FakeDevice {
    scenario: &'static str,
    clock: Arc<dyn Clock>,
    reachable: watch::Receiver<bool>,
}

/// Makes a fake device unreachable or reachable again.
#[derive(Clone)]
pub struct FakeControls(Arc<watch::Sender<bool>>);

impl FakeControls {
    pub fn set_reachable(&self, reachable: bool) {
        self.0.send_replace(reachable);
    }
}

impl FakeDevice {
    pub fn new(
        scenario: &str,
        clock: Arc<dyn Clock>,
    ) -> Result<(Self, FakeControls), UnknownScenario> {
        let scenario = SCENARIOS
            .into_iter()
            .find(|known| *known == scenario)
            .ok_or_else(|| UnknownScenario(scenario.to_owned()))?;
        let (sender, reachable) = watch::channel(true);
        Ok((
            Self {
                scenario,
                clock,
                reachable,
            },
            FakeControls(Arc::new(sender)),
        ))
    }
}

impl DeviceMonitor for FakeDevice {
    fn snapshots(&self) -> BoxStream<'static, DeviceSnapshot> {
        let scenario = self.scenario;
        let clock = Arc::clone(&self.clock);
        let mut reachable = self.reachable.clone();
        reachable.mark_changed();
        stream::unfold(
            (reachable, clock),
            move |(mut reachable, clock)| async move {
                reachable.changed().await.ok()?;
                let online = *reachable.borrow_and_update();
                let mut snapshot = scenario_snapshot(scenario, clock.as_ref());
                snapshot.online = snapshot.online && online;
                Some((snapshot, (reachable, clock)))
            },
        )
        .boxed()
    }
}

fn scenario_snapshot(scenario: &str, clock: &dyn Clock) -> DeviceSnapshot {
    let mut snapshot = DeviceSnapshot {
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin: Origin::Service {
            id: "Qobuz".into(),
            name: "Qobuz".into(),
            cast: false,
        },
        content: Some(Content {
            track: Track {
                title: "Example Song".into(),
                artists: vec!["Example Artist".into()],
                album: Some("Example Album".into()),
            },
            artwork: Some(cover("Example Album", 210)),
            position: Some(Position {
                elapsed: Duration::from_secs(42),
                duration: Some(Duration::from_secs(214)),
                observed_at: clock.now(),
            }),
            quality: Some(
                AudioQuality::format(AudioFormat {
                    codec: "FLAC".into(),
                    bit_depth: 16,
                    sample_rate_hz: 44_100,
                })
                .with_detail("CD quality"),
            ),
            queue: QueueView::Available(
                [
                    "Second Song",
                    "Third Song",
                    "Fourth Song",
                    "Fifth Song",
                    "Sixth Song",
                ]
                .iter()
                .map(|title| QueueEntry {
                    title: (*title).into(),
                    artists: vec!["Example Artist".into()],
                    artwork: Some(cover("Example Album", 210)),
                })
                .collect(),
            ),
            service: None,
            palette: None,
        }),
    };
    let content = snapshot.content.as_mut();

    match (scenario, content) {
        ("cast-without-queue", Some(content)) => {
            snapshot.origin = Origin::Service {
                id: "Spotify".into(),
                name: "Spotify".into(),
                cast: true,
            };
            content.track.title = "Cast Song".into();
            content.artwork = Some(cover("Cast Album", 140));
            content.quality = Some(AudioQuality::bitrate(160_000));
            content.queue = QueueView::NotApplicable;
        }
        ("long-title", Some(content)) => {
            content.track.title =
                "A Remarkably Long Song Title That Keeps Going Well Past Two Lines On Any Reasonable Screen (Extended Live Version)".into();
            content.track.artists = vec![
                "An Artist With A Rather Long Name".into(),
                "Featured Guest".into(),
            ];
            content.track.album =
                Some("An Album Title That Is Also Far Too Long To Fit On One Line".into());
        }
        ("no-artwork", Some(content)) => {
            content.artwork = None;
        }
        ("hi-res", Some(content)) => {
            content.quality = Some(AudioQuality::format(AudioFormat {
                codec: "FLAC".into(),
                bit_depth: 24,
                sample_rate_hz: 96_000,
            }));
            content.artwork = Some(cover("Hi-Res Album", 30));
        }
        ("paused", _) => snapshot.state = PlaybackState::Paused,
        ("stopped", _) => snapshot.state = PlaybackState::Stopped,
        ("offline", _) => snapshot.online = false,
        _ => {}
    }
    snapshot
}

/// A generated square cover: a gradient with the initials of `title`.
fn cover(title: &str, hue: u16) -> ArtworkRef {
    let initials: String = title
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect();
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="1000" viewBox="0 0 1000 1000"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="hsl({hue},65%,55%)"/><stop offset="1" stop-color="hsl({},60%,25%)"/></linearGradient></defs><rect width="1000" height="1000" fill="url(#g)"/><text x="500" y="560" font-family="sans-serif" font-size="260" font-weight="700" fill="rgba(255,255,255,0.85)" text-anchor="middle">{initials}</text></svg>"#,
        (hue + 40) % 360
    );
    ArtworkRef::new(format!("data:image/svg+xml;base64,{}", BASE64.encode(svg)))
}
