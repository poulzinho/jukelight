//! The published JSON shapes (API v1). Separate from domain types, so the
//! domain can change without breaking the kiosk.

use jukelight_application::{ArtworkSource, BrandLogos};
use jukelight_domain::{
    ArtworkRef, AudioInput, AudioQuality, Brand, InputKind, PlaybackState, PlayerSnapshot,
    QualityMeasure, QueueView, Rgb, Source, Timestamp,
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
    pub device: DeviceDto,
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
        device: DeviceDto {
            brand: brand(snapshot.maker.as_ref()),
            name: snapshot.player_name.clone(),
            logo_url: logo_url(snapshot.maker.as_ref(), logos),
        },
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
        // As the label writes it: sources report `flac` or `FLAC`.
        codec: format.map(|f| f.codec.to_uppercase()),
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
