//! A fake device with scripted scenarios, for kiosk work, demos and tests
//! without a real player. Selected with `JUKELIGHT_DEVICE=fake:<scenario>`.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures::StreamExt;
use futures::stream::{self, BoxStream};
use jukelight_application::{Clock, DeviceMonitor, SessionProvider};
use jukelight_domain::{
    ArtworkRef, AudioFormat, AudioInput, AudioQuality, Brand, Change, Content, Decibels, Delivery,
    DeviceSnapshot, InputKind, Origin, Output, OutputKind, Palette, PlaybackState, Position,
    QueueEntry, QueueView, Rgb, Service, Speaker, SpeakerKind, Timestamp, Track,
};
use tokio::sync::watch;

pub const SCENARIOS: [&str; 23] = [
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
    "music-assistant",
    "music-assistant-unavailable",
    "long-signal",
    "upsampled",
    "unknown-quality",
    "group",
    "group-dropped",
    "group-many",
    "group-long-names",
    "airplay",
    "bluetooth",
    "radio",
];

/// Scenarios that a fake Music Assistant drives: it claims the stream.
const MUSIC_ASSISTANT: [&str; 8] = [
    "music-assistant",
    "music-assistant-unavailable",
    "long-signal",
    "upsampled",
    "group",
    "group-dropped",
    "group-many",
    "group-long-names",
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
        maker: Brand::new("bluesound"),
        // The example player has its subwoofer on.
        outputs: vec![Output {
            name: None,
            kind: OutputKind::Subwoofer,
        }],
        name: "Example Player".into(),
        player_key: "aabbccddeeff".into(),
        online: true,
        state: PlaybackState::Playing,
        origin: Origin::Service {
            brand: Brand::new("qobuz"),
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
            delivery: None,
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
            speakers: Vec::new(),
            held: false,
        }),
    }
}

fn scenario_snapshot(scenario: &str, clock: &dyn Clock) -> DeviceSnapshot {
    let mut snapshot = example_snapshot(clock);
    let content = snapshot.content.as_mut();

    match (scenario, content) {
        ("cast-without-queue", Some(content)) => {
            snapshot.origin = Origin::Service {
                brand: Brand::new("spotify"),
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
        (scenario, _) if MUSIC_ASSISTANT.contains(&scenario) => {
            // Music Assistant streams to the player; the fake provider knows
            // the track.
            snapshot.origin = Origin::Stream {
                url: "http://192.168.1.50:8097/flow/session1/aabbccddeeff/1.flac".into(),
            };
            snapshot.content = None;
        }
        ("hi-res", Some(content)) => {
            content.quality = Some(AudioQuality::format(AudioFormat {
                codec: "FLAC".into(),
                bit_depth: 24,
                sample_rate_hz: 96_000,
            }));
            content.artwork = cover(30);
        }
        ("airplay" | "bluetooth", Some(content)) => {
            let (id, name) = if scenario == "airplay" {
                ("AirPlay", "AirPlay")
            } else {
                ("Bluetooth", "Bluetooth")
            };
            snapshot.origin = Origin::Service {
                brand: Brand::new(&id.to_lowercase()),
                id: id.into(),
                name: name.into(),
                cast: true,
            };
            content.track.title = "Phone Song".into();
            content.quality = None;
            content.queue = QueueView::NotApplicable;
        }
        ("radio", Some(content)) => {
            snapshot.origin = Origin::Service {
                brand: None,
                id: "TuneIn".into(),
                name: "Example FM".into(),
                cast: false,
            };
            content.track.title = "Radio Show".into();
            content.quality = Some(AudioQuality::bitrate(128_000));
            content.queue = QueueView::NotApplicable;
        }
        ("unknown-quality", Some(content)) => {
            content.quality = None;
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

/// A fake Music Assistant for the scenarios it drives: it claims the
/// player's stream and plays a Hi-Res Qobuz track, normalised (for
/// `long-signal` also resampled, with DSP and a long detail), a CD-quality
/// Subsonic track it upsamples (`upsampled`), or can't be reached. `None` for the other
/// scenarios.
pub fn music_assistant(
    scenario: &str,
    clock: &dyn Clock,
) -> Option<std::sync::Arc<dyn SessionProvider>> {
    if !MUSIC_ASSISTANT.contains(&scenario) {
        return None;
    }
    let content = (scenario != "music-assistant-unavailable").then(|| {
        let mut content = scenario_snapshot("playing-with-queue", clock)
            .content
            .unwrap_or_else(|| unreachable!("the example always has content"));
        let (service, source, delivered, changes, detail) = match scenario {
            "upsampled" => (
                Service::new(Brand::new("subsonic"), "Subsonic"),
                flac(16, 44_100),
                flac(24, 96_000),
                vec![Change::Resampled],
                "Lossless · → 24/96 to player",
            ),
            "long-signal" => (
                Service::new(Brand::new("qobuz"), "Qobuz"),
                flac(24, 192_000),
                flac(24, 96_000),
                vec![
                    Change::Resampled,
                    Change::Normalised {
                        gain: Some(Decibels::from_db(-7.4)),
                    },
                    Change::Dsp,
                ],
                "Normalised, crossfade 6 s, dynamic range 12 dB, track gain −7.4 dB",
            ),
            _ => (
                Service::new(Brand::new("qobuz"), "Qobuz"),
                flac(24, 96_000),
                flac(24, 96_000),
                vec![Change::Normalised {
                    gain: Some(Decibels::from_db(-6.2)),
                }],
                "Normalised",
            ),
        };
        content.quality = Some(source.with_detail(detail));
        content.delivery = Some(Delivery {
            quality: Some(delivered),
            changes,
        });
        content.service = Some(service);
        content.speakers = group_speakers(scenario);
        content
    });
    Some(std::sync::Arc::new(FakeMusicAssistant { content }))
}

/// The other speakers of a group scenario; example names only.
fn group_speakers(scenario: &str) -> Vec<Speaker> {
    let speaker = |name: &str, kind, playing, quality| Speaker {
        name: name.into(),
        kind,
        playing,
        quality,
    };
    let pcm = || {
        Some(AudioQuality::format(AudioFormat {
            codec: "s16le".into(),
            bit_depth: 16,
            sample_rate_hz: 44_100,
        }))
    };
    match scenario {
        "group" => vec![
            speaker("Kitchen", SpeakerKind::Speaker, true, pcm()),
            speaker(
                "Office",
                SpeakerKind::Computer,
                true,
                Some(flac(24, 44_100)),
            ),
        ],
        "group-dropped" => vec![
            speaker("Kitchen", SpeakerKind::Speaker, true, pcm()),
            speaker("Office", SpeakerKind::Computer, false, None),
        ],
        "group-many" => vec![
            speaker("Kitchen", SpeakerKind::Speaker, true, pcm()),
            speaker(
                "Garden",
                SpeakerKind::Speaker,
                true,
                Some(AudioQuality::bitrate(256_000)),
            ),
            speaker(
                "Office",
                SpeakerKind::Computer,
                true,
                Some(flac(24, 44_100)),
            ),
            speaker("Bedroom", SpeakerKind::Speaker, true, pcm()),
            speaker("Hall", SpeakerKind::Speaker, true, pcm()),
        ],
        "group-long-names" => vec![
            speaker(
                "Upstairs Bathroom Speaker",
                SpeakerKind::Speaker,
                true,
                Some(flac(24, 96_000)),
            ),
            speaker("Guest Room Bookshelf", SpeakerKind::Speaker, true, pcm()),
        ],
        _ => Vec::new(),
    }
}

fn flac(bit_depth: u32, sample_rate_hz: u32) -> AudioQuality {
    AudioQuality::format(AudioFormat {
        codec: "flac".into(),
        bit_depth,
        sample_rate_hz,
    })
}

struct FakeMusicAssistant {
    content: Option<Content>,
}

impl SessionProvider for FakeMusicAssistant {
    fn label(&self) -> &'static str {
        "Music Assistant"
    }

    fn brand(&self) -> Option<Brand> {
        Brand::new("music-assistant")
    }

    fn claims(&self, device: &DeviceSnapshot) -> bool {
        matches!(device.origin, Origin::Stream { .. })
    }

    fn content(&self) -> Option<Content> {
        self.content.clone()
    }

    fn follow(&self, _: &DeviceSnapshot) {}

    fn updates(&self) -> BoxStream<'static, ()> {
        stream::pending().boxed()
    }
}
