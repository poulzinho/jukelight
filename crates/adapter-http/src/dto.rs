//! The published JSON shapes (API v1). Separate from domain types, so the
//! domain can change without breaking the kiosk.

use jukelight_application::{ArtworkSource, BrandLogos};
use jukelight_domain::{
    ArtworkRef, AudioInput, AudioQuality, Brand, Change, Content, Decibels, InputKind, OutputKind,
    PlaybackState, PlayerSnapshot, QualityClass, QualityMeasure, QueueView, Rgb, Source, Speaker,
    SpeakerKind, Timestamp, is_pcm, signal_class,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDto {
    pub online: bool,
    pub device_name: String,
    pub state: &'static str,
    pub session: SessionDto,
    pub track: Option<TrackDto>,
    /// The large version of the track's cover; `None` (`null`) without one.
    pub art_url: Option<String>,
    /// A tiny version of the same cover, to stretch over the screen; `None`
    /// (`null`) without a cover.
    pub backdrop_url: Option<String>,
    pub position: Option<PositionDto>,
    pub quality: Option<QualityDto>,
    /// `None` when the session has no queue.
    pub queue: Option<Vec<QueueEntryDto>>,
    pub palette: Option<PaletteDto>,
    pub input: Option<InputDto>,
    /// Where the music comes from; `None` (`null`) when nothing is known.
    pub source: Option<SourceDto>,
    /// How the music reaches the player; `None` (`null`) without content
    /// (idle, or an input).
    pub signal: Option<SignalDto>,
    pub device: DeviceDto,
    /// The other speakers playing the same session; empty when the player
    /// plays alone.
    pub speakers: Vec<SpeakerDto>,
    /// What the player itself drives: its output speaker, its subwoofer.
    pub outputs: Vec<OutputDto>,
}

/// Another speaker of the session.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerDto {
    pub name: String,
    /// `speaker` or `computer`.
    pub kind: &'static str,
    pub playing: bool,
    /// The format it receives, with its class; `None` (`null`) when unknown.
    pub quality: Option<SpeakerQualityDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerQualityDto {
    #[serde(flatten)]
    pub quality: QualityDto,
    /// `hi-res`, `lossless` or `lossy`.
    pub class: &'static str,
}

/// Something the player drives.
#[derive(Debug, Serialize)]
pub struct OutputDto {
    pub name: String,
    /// `speaker` or `subwoofer`.
    pub kind: &'static str,
}

/// The path from the source, through the route, to the player.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalDto {
    /// `hi-res`, `lossless`, `lossy` or `unknown`: the lowest along the path.
    pub class: &'static str,
    /// The quality as the source delivers it.
    pub source: Option<QualityDto>,
    /// The quality the route sends to the player; `None` (`null`) without a
    /// route or when it isn't reported.
    pub delivered: Option<QualityDto>,
    pub changes: Vec<ChangeDto>,
    /// The user's Hi-Res logo, when the whole path is Hi-Res.
    pub hi_res_logo_url: Option<String>,
}

/// One thing the route changed on the way.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChangeDto {
    Resampled,
    BitDepth,
    #[serde(rename_all = "camelCase")]
    Normalised {
        gain_db: Option<f64>,
    },
    Dsp,
}

/// A music service or another source.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDto {
    /// A short identifier such as `qobuz`; `None` (`null`) when unknown.
    pub brand: Option<String>,
    pub name: String,
    /// The user's logo for the brand; `None` (`null`) without one.
    pub logo_url: Option<String>,
    /// The route the music takes, such as Music Assistant.
    pub via: Option<RouteDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDto {
    pub brand: Option<String>,
    pub name: String,
    pub available: bool,
    pub logo_url: Option<String>,
}

/// The player itself: its maker's brand and its name.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDto {
    pub brand: Option<String>,
    pub name: String,
    pub logo_url: Option<String>,
    /// The maker's mark (`<maker>-mark`), when supplied.
    pub mark_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionDto {
    pub label: String,
    pub available: bool,
}

#[derive(Debug, Serialize)]
pub struct TrackDto {
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionDto {
    /// Seconds.
    pub elapsed: f64,
    /// Seconds.
    pub duration: Option<f64>,
    /// Milliseconds since the Unix epoch.
    pub observed_at: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityDto {
    pub label: String,
    pub detail: Option<String>,
    pub hi_res: bool,
    /// The user's Hi-Res logo, for a Hi-Res quality; `None` (`null`) otherwise.
    pub hi_res_logo_url: Option<String>,
    /// For a lossless format; `None` (`null`) for a lossy one.
    pub codec: Option<String>,
    pub bit_depth: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    /// Bits per second, for a lossy stream; `None` (`null`) otherwise.
    pub bitrate: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueEntryDto {
    pub title: String,
    pub artists: Vec<String>,
    pub art_url: Option<String>,
}

/// Album colours, each as `[red, green, blue]` from 0 to 255.
#[derive(Debug, Serialize)]
pub struct PaletteDto {
    pub background: [u8; 3],
    pub accent: [u8; 3],
}

/// An audio input while it plays.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputDto {
    /// `optical`, `analog`, `hdmi-arc` or `other`.
    pub kind: &'static str,
    pub name: String,
    /// When its signal started (milliseconds since the Unix epoch); `None`
    /// while the player waits for a signal.
    pub started_at: Option<u64>,
}

pub fn player_dto(
    snapshot: &PlayerSnapshot,
    artwork: &dyn ArtworkSource,
    logos: &dyn BrandLogos,
) -> PlayerDto {
    let content = snapshot.content.as_ref();
    let art_url = |art: Option<&ArtworkRef>, size: &str| {
        art.map(|a| format!("/api/v1/art/{}{size}", artwork.issue(a).as_str()))
    };
    let cover = content.and_then(|c| c.artwork.as_ref());

    PlayerDto {
        online: snapshot.online,
        device_name: snapshot.player_name.clone(),
        state: state(snapshot.state),
        session: SessionDto {
            label: snapshot.session.label.clone(),
            available: snapshot.session.available,
        },
        track: content.map(|c| TrackDto {
            title: c.track.title.clone(),
            artists: c.track.artists.clone(),
            album: c.track.album.clone(),
        }),
        art_url: art_url(cover, "?size=large"),
        backdrop_url: art_url(cover, "?size=backdrop"),
        position: content.and_then(|c| c.position).map(|p| PositionDto {
            elapsed: p.elapsed.as_secs_f64(),
            duration: p.duration.map(|d| d.as_secs_f64()),
            observed_at: p.observed_at.as_millis(),
        }),
        quality: content
            .and_then(|c| c.quality.as_ref())
            .map(|quality| quality_dto(quality, logos)),
        queue: content.and_then(|c| match &c.queue {
            QueueView::Available(entries) => Some(
                entries
                    .iter()
                    .map(|e| QueueEntryDto {
                        title: e.title.clone(),
                        artists: e.artists.clone(),
                        art_url: art_url(e.artwork.as_ref(), ""),
                    })
                    .collect(),
            ),
            QueueView::NotApplicable => None,
        }),
        palette: content.and_then(|c| c.palette).map(|p| PaletteDto {
            background: rgb(p.background),
            accent: rgb(p.accent),
        }),
        input: snapshot.input.as_ref().map(input_dto),
        source: snapshot
            .source
            .as_ref()
            .map(|source| source_dto(source, logos)),
        signal: content.map(|content| signal_dto(content, logos)),
        device: DeviceDto {
            brand: brand(snapshot.maker.as_ref()),
            name: snapshot.player_name.clone(),
            logo_url: logo_url(snapshot.maker.as_ref(), logos),
            mark_url: snapshot
                .maker
                .as_ref()
                .and_then(|maker| Brand::new(&format!("{}-mark", maker.as_str())))
                .and_then(|mark| logo_url(Some(&mark), logos)),
        },
        speakers: content
            .map(|c| {
                c.speakers
                    .iter()
                    .map(|speaker| speaker_dto(speaker, c.quality.as_ref(), logos))
                    .collect()
            })
            .unwrap_or_default(),
        outputs: snapshot
            .outputs
            .iter()
            .map(|output| OutputDto {
                name: output.display_name().to_owned(),
                kind: match output.kind {
                    OutputKind::Speaker => "speaker",
                    OutputKind::Subwoofer => "subwoofer",
                },
            })
            .collect(),
    }
}

fn brand(brand: Option<&Brand>) -> Option<String> {
    brand.map(|brand| brand.as_str().to_owned())
}

/// The URL of `brand`'s logo, when the user supplied one.
fn logo_url(brand: Option<&Brand>, logos: &dyn BrandLogos) -> Option<String> {
    let brand = brand?;
    let version = logos.version(brand)?;
    Some(format!("/api/v1/logos/{}?v={version}", brand.as_str()))
}

fn source_dto(source: &Source, logos: &dyn BrandLogos) -> SourceDto {
    SourceDto {
        brand: brand(source.service.brand.as_ref()),
        name: source.service.name.clone(),
        logo_url: logo_url(source.service.brand.as_ref(), logos),
        via: source.via.as_ref().map(|route| RouteDto {
            brand: brand(route.service.brand.as_ref()),
            name: route.service.name.clone(),
            available: route.available,
            logo_url: logo_url(route.service.brand.as_ref(), logos),
        }),
    }
}

fn speaker_dto(
    speaker: &Speaker,
    source: Option<&AudioQuality>,
    logos: &dyn BrandLogos,
) -> SpeakerDto {
    SpeakerDto {
        name: speaker.name.clone(),
        kind: match speaker.kind {
            SpeakerKind::Speaker => "speaker",
            SpeakerKind::Computer => "computer",
        },
        playing: speaker.playing,
        quality: speaker.quality.as_ref().map(|quality| SpeakerQualityDto {
            quality: quality_dto(quality, logos),
            class: class_name(speaker.class_from(source)),
        }),
    }
}

fn class_name(class: QualityClass) -> &'static str {
    match class {
        QualityClass::HiRes => "hi-res",
        QualityClass::Lossless => "lossless",
        QualityClass::Lossy => "lossy",
        QualityClass::Unknown => "unknown",
    }
}

fn signal_dto(content: &Content, logos: &dyn BrandLogos) -> SignalDto {
    let delivery = content.delivery.as_ref();
    let class = signal_class(content.quality.as_ref(), delivery);
    SignalDto {
        class: match class {
            QualityClass::HiRes => "hi-res",
            QualityClass::Lossless => "lossless",
            QualityClass::Lossy => "lossy",
            QualityClass::Unknown => "unknown",
        },
        source: content
            .quality
            .as_ref()
            .map(|quality| quality_dto(quality, logos)),
        delivered: delivery
            .and_then(|delivery| delivery.quality.as_ref())
            .map(|quality| quality_dto(quality, logos)),
        changes: delivery
            .map(|delivery| delivery.changes.iter().copied().map(change_dto).collect())
            .unwrap_or_default(),
        hi_res_logo_url: if class == QualityClass::HiRes {
            logo_url(Brand::new("hi-res").as_ref(), logos)
        } else {
            None
        },
    }
}

fn change_dto(change: Change) -> ChangeDto {
    match change {
        Change::Resampled => ChangeDto::Resampled,
        Change::BitDepth => ChangeDto::BitDepth,
        Change::Normalised { gain } => ChangeDto::Normalised {
            gain_db: gain.map(Decibels::as_db),
        },
        Change::Dsp => ChangeDto::Dsp,
    }
}

fn quality_dto(quality: &AudioQuality, logos: &dyn BrandLogos) -> QualityDto {
    let hi_res = quality.is_hi_res();
    let (format, bitrate) = match &quality.measure {
        QualityMeasure::Format(format) => (Some(format), None),
        QualityMeasure::Bitrate { bits_per_second } => (None, Some(*bits_per_second)),
    };
    QualityDto {
        label: quality.label(),
        detail: quality.detail.clone(),
        hi_res,
        hi_res_logo_url: if hi_res {
            logo_url(Brand::new("hi-res").as_ref(), logos)
        } else {
            None
        },
        // As the label writes it: sources report `flac` or `FLAC`; raw
        // samples (`s16le`, `f32le`, …) are PCM.
        codec: format.map(|f| {
            if is_pcm(&f.codec) {
                "PCM".to_owned()
            } else {
                f.codec.to_uppercase()
            }
        }),
        bit_depth: format.map(|f| f.bit_depth),
        sample_rate_hz: format.map(|f| f.sample_rate_hz),
        bitrate,
    }
}

fn input_dto(input: &AudioInput) -> InputDto {
    InputDto {
        kind: match input.kind {
            InputKind::Optical => "optical",
            InputKind::Analog => "analog",
            InputKind::HdmiArc => "hdmi-arc",
            InputKind::Other => "other",
        },
        name: input.name.clone(),
        started_at: input.started_at.map(Timestamp::as_millis),
    }
}

fn rgb(Rgb(red, green, blue): Rgb) -> [u8; 3] {
    [red, green, blue]
}

fn state(state: PlaybackState) -> &'static str {
    match state {
        PlaybackState::Playing => "playing",
        PlaybackState::Paused => "paused",
        PlaybackState::Stopped => "stopped",
        PlaybackState::Buffering => "buffering",
    }
}
