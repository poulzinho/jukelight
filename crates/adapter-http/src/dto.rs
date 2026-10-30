//! The published JSON shapes (API v1). Separate from domain types, so the
//! domain can change without breaking the kiosk.

use jukelight_application::ArtworkSource;
use jukelight_domain::{ArtworkRef, PlaybackState, PlayerSnapshot, QueueView};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDto {
    pub online: bool,
    pub device_name: String,
    pub state: &'static str,
    pub session: SessionDto,
    pub track: Option<TrackDto>,
    pub art_url: Option<String>,
    pub position: Option<PositionDto>,
    pub quality: Option<QualityDto>,
    /// `None` when the session has no queue.
    pub queue: Option<Vec<QueueEntryDto>>,
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

pub fn player_dto(snapshot: &PlayerSnapshot, artwork: &dyn ArtworkSource) -> PlayerDto {
    let content = snapshot.content.as_ref();
    let art_url = |art: Option<&ArtworkRef>| {
        art.map(|a| format!("/api/v1/art/{}", artwork.issue(a).as_str()))
    };

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
        art_url: content.and_then(|c| art_url(c.artwork.as_ref())),
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
                        art_url: art_url(e.artwork.as_ref()),
                    })
                    .collect(),
            ),
            QueueView::NotApplicable => None,
        }),
    }
}

fn state(state: PlaybackState) -> &'static str {
    match state {
        PlaybackState::Playing => "playing",
        PlaybackState::Paused => "paused",
        PlaybackState::Stopped => "stopped",
        PlaybackState::Buffering => "buffering",
    }
}
