//! The published JSON shapes (API v1). Separate from domain types, so the
//! domain can change without breaking the kiosk.

use jukelight_application::ArtworkSource;
use jukelight_domain::{
    ArtworkRef, AudioInput, InputKind, PlaybackState, PlayerSnapshot, QueueView, Rgb, Timestamp,
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

pub fn player_dto(snapshot: &PlayerSnapshot, artwork: &dyn ArtworkSource) -> PlayerDto {
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
            .map(|q| QualityDto {
                label: q.label(),
                detail: q.detail.clone(),
                hi_res: q.is_hi_res(),
            }),
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
