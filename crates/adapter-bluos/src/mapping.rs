//! Translation of BluOS fields into Jukelight's device snapshot. This is the
//! anti-corruption layer: no BluOS term leaves this module.

use std::time::Duration;

use jukelight_domain::{
    ArtworkRef, AudioFormat, AudioQuality, Content, DeviceSnapshot, Origin, PlaybackState,
    Position, QueueEntry, QueueView, Timestamp, Track,
};

use crate::parse::{Identity, PlaylistEntry, Status};

/// The device snapshot for one `/Status` response.
///
/// `playlist` is the current `/Playlist`; it is used only for native,
/// non-cast sessions, because BluOS keeps stale entries during cast sessions.
pub fn to_device_snapshot(
    status: &Status,
    identity: &Identity,
    playlist: &[PlaylistEntry],
    base_url: &str,
    now: Timestamp,
) -> DeviceSnapshot {
    let origin = origin(status);
    DeviceSnapshot {
        name: identity.name.clone(),
        player_key: identity.player_key.clone(),
        online: true,
        state: playback_state(status.get("state")),
        content: content(status, &origin, playlist, base_url, now),
        origin,
    }
}

/// Whether `/Playlist` holds this session's queue.
pub fn is_native(origin: &Origin) -> bool {
    matches!(origin, Origin::Service { cast: false, .. })
}

fn playback_state(state: Option<&str>) -> PlaybackState {
    match state {
        // Cast sessions and streams report `stream` while playing.
        Some("play" | "stream") => PlaybackState::Playing,
        Some("pause") => PlaybackState::Paused,
        Some("connecting") => PlaybackState::Buffering,
        _ => PlaybackState::Stopped,
    }
}

fn origin(status: &Status) -> Origin {
    if let Some(url) = status.get("streamUrl").filter(|url| is_http(url)) {
        return Origin::Stream {
            url: url.to_owned(),
        };
    }
    match status.get("service") {
        Some(id) => Origin::Service {
            id: id.to_owned(),
            name: status.get("serviceName").unwrap_or(id).to_owned(),
            cast: status.get("serviceType") == Some("Endpoint"),
        },
        None => Origin::None,
    }
}

fn content(
    status: &Status,
    origin: &Origin,
    playlist: &[PlaylistEntry],
    base_url: &str,
    now: Timestamp,
) -> Option<Content> {
    let title = status.get("title1")?;
    let track = Track {
        title: title.to_owned(),
        artists: status
            .get("artist")
            .or(status.get("title2"))
            .map(|a| vec![a.to_owned()])
            .unwrap_or_default(),
        album: status
            .get("album")
            .or(status.get("title3"))
            .map(str::to_owned),
    };
    let artwork = status
        .get("image")
        .or(status.get("currentImage"))
        .map(|path| ArtworkRef::new(absolute(path, base_url)));

    Some(Content {
        track,
        artwork,
        position: position(status, origin, now),
        quality: quality(status, origin),
        queue: queue(status, origin, playlist, base_url),
        service: None,
        palette: None,
    })
}

/// `secs` counts from the start of the track only for service sessions; for
/// a stream it counts from the start of the stream.
fn position(status: &Status, origin: &Origin, now: Timestamp) -> Option<Position> {
    if !matches!(origin, Origin::Service { .. }) {
        return None;
    }
    let elapsed = status.get("secs")?.parse().ok().map(Duration::from_secs)?;
    let duration = status
        .get("totlen")
        .and_then(|t| t.parse().ok())
        .map(Duration::from_secs);
    Some(Position {
        elapsed,
        duration,
        observed_at: now,
    })
}

/// Only for service sessions: for a stream, `streamFormat` describes what is
/// sent to the player (possibly converted, e.g. 16 to 24 bits), not the source.
fn quality(status: &Status, origin: &Origin) -> Option<AudioQuality> {
    if !matches!(origin, Origin::Service { .. }) {
        return None;
    }
    let detail = match status.get("quality") {
        Some("cd") => Some("CD quality"),
        Some("mqa" | "mqaAuthored") => Some("MQA"),
        _ => None,
    };
    let quality = if let Some(text) = status.get("streamFormat") {
        AudioQuality::format(parse_stream_format(text)?)
    } else {
        // Without a format, a numeric `quality` is the bitrate (e.g. Spotify).
        let bitrate: u32 = status.get("quality")?.parse().ok()?;
        (bitrate > 0).then(|| AudioQuality::bitrate(bitrate))?
    };
    Some(match detail {
        Some(detail) => quality.with_detail(detail),
        None => quality,
    })
}

/// "FLAC 24/96" or "FLAC 16/44.1".
fn parse_stream_format(text: &str) -> Option<AudioFormat> {
    let (codec, rest) = text.split_once(' ')?;
    let (bits, kilohertz) = rest.trim().split_once('/')?;
    let (whole, fraction) = kilohertz.split_once('.').unwrap_or((kilohertz, ""));
    let fraction = format!("{fraction:0<3}");
    let sample_rate_hz =
        whole.parse::<u32>().ok()? * 1000 + fraction.get(..3)?.parse::<u32>().ok()?;
    Some(AudioFormat {
        codec: codec.to_owned(),
        bit_depth: bits.parse().ok()?,
        sample_rate_hz,
    })
}

fn queue(
    status: &Status,
    origin: &Origin,
    playlist: &[PlaylistEntry],
    base_url: &str,
) -> QueueView {
    if !is_native(origin) {
        return QueueView::NotApplicable;
    }
    let current: usize = status.get("song").and_then(|s| s.parse().ok()).unwrap_or(0);
    QueueView::Available(
        playlist
            .iter()
            .skip(current + 1)
            .map(|entry| QueueEntry {
                title: entry.title.clone(),
                artists: entry.artist.iter().cloned().collect(),
                artwork: entry
                    .image
                    .as_deref()
                    .map(|path| ArtworkRef::new(absolute(path, base_url))),
            })
            .collect(),
    )
}

fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn absolute(path: &str, base_url: &str) -> String {
    if is_http(path) {
        path.to_owned()
    } else if path.starts_with('/') {
        format!("{base_url}{path}")
    } else {
        format!("{base_url}/{path}")
    }
}
