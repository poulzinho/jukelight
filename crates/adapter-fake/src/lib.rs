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
    ArtworkRef, AudioFormat, AudioInput, AudioQuality, Content, DeviceSnapshot, InputKind, Origin,
    Palette, PlaybackState, Position, QueueEntry, QueueView, Rgb, Timestamp, Track,
};
use tokio::sync::watch;

pub const SCENARIOS: [&str; 11] = [
    "playing-with-queue",
    "cast-without-queue",
    "long-title",
    "no-album",
    "no-artwork",
    "hi-res",
    "album-colours",
    "optical-input",
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

/// A playing track with everything known, which the scenarios vary.
fn example_snapshot(clock: &dyn Clock) -> DeviceSnapshot {
    DeviceSnapshot {
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
            artwork: cover(210),
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
                    artwork: cover(210),
                })
                .collect(),
            ),
            service: None,
            palette: None,
        }),
    }
}

fn scenario_snapshot(scenario: &str, clock: &dyn Clock) -> DeviceSnapshot {
    let mut snapshot = example_snapshot(clock);
    let content = snapshot.content.as_mut();

    match (scenario, content) {
        ("cast-without-queue", Some(content)) => {
            snapshot.origin = Origin::Service {
                id: "Spotify".into(),
                name: "Spotify".into(),
                cast: true,
            };
            content.track.title = "Cast Song".into();
            content.artwork = cover(140);
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
        ("no-album", Some(content)) => {
            content.track.title = "Single Song".into();
            content.track.album = None;
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
            content.artwork = cover(30);
        }
        ("album-colours", Some(content)) => {
            content.artwork = cover(20);
            content.palette = Some(Palette {
                background: Rgb(73, 51, 38),
                accent: Rgb(209, 147, 108),
            });
        }
        ("optical-input", _) => (snapshot.origin, snapshot.content) = (optical_input(clock), None),
        ("paused", _) => snapshot.state = PlaybackState::Paused,
        ("stopped", _) => snapshot.state = PlaybackState::Stopped,
        ("offline", _) => snapshot.online = false,
        _ => {}
    }
    snapshot
}

/// An optical input streaming for 23 minutes: no track, only the input.
fn optical_input(clock: &dyn Clock) -> Origin {
    Origin::Input(AudioInput {
        kind: InputKind::Optical,
        name: "Example Input".into(),
        started_at: Some(Timestamp::from_millis(
            clock.now().as_millis().saturating_sub(23 * 60_000),
        )),
    })
}

/// A generated square cover: a diagonal gradient around `hue`, as a PNG data
/// URL. A raster image, so the backdrop can be made from it too. `None` if
/// it can't be encoded: no cover rather than an empty image.
fn cover(hue: u16) -> Option<ArtworkRef> {
    const SIZE: u32 = 256;
    let from = hsl(hue, 0.65, 0.55);
    let to = hsl((hue + 40) % 360, 0.60, 0.25);
    let image = image::RgbImage::from_fn(SIZE, SIZE, |x, y| {
        let t = f64::from(x + y) / f64::from(2 * (SIZE - 1));
        image::Rgb(std::array::from_fn(|i| {
            let value = f64::from(from[i]) + (f64::from(to[i]) - f64::from(from[i])) * t;
            // The mix of two bytes stays a byte.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let byte = value.round().clamp(0.0, 255.0) as u8;
            byte
        }))
    });
    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some(ArtworkRef::new(format!(
        "data:image/png;base64,{}",
        BASE64.encode(png)
    )))
}

/// An HSL colour (hue in degrees, saturation and lightness from 0 to 1) as RGB.
fn hsl(hue: u16, saturation: f64, lightness: f64) -> [u8; 3] {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = f64::from(hue) / 60.0;
    let second = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let (r, g, b) = match hue / 60 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let offset = lightness - chroma / 2.0;
    [r, g, b].map(|channel| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let byte = ((channel + offset) * 255.0).round().clamp(0.0, 255.0) as u8;
        byte
    })
}
