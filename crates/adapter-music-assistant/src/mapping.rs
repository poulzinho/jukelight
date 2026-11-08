//! Turns Music Assistant's responses into session content. Pure: no IO.

use std::time::Duration;

use jukelight_domain::{
    ArtworkRef, AudioFormat, AudioQuality, Brand, Change, Content, Decibels, Delivery, Palette,
    Position, QueueEntry, QueueView, Rgb, Service, Timestamp, Track,
};

use crate::api;

/// Entries shown as up next.
pub const UP_NEXT: usize = 5;
/// Beyond this distance from the local clock, Music Assistant's measurement
/// time is not trusted and the position counts as observed now.
const MAX_CLOCK_SKEW: Duration = Duration::from_secs(10);
/// Normalisation modes that change the audio; `measurement_only` and
/// `disabled` do not.
const ALTERS_VOLUME: &[&str] = &[
    "dynamic",
    "fixed_gain",
    "fallback_fixed_gain",
    "fallback_dynamic",
    "source",
];
/// Provider instances that stand for Music Assistant itself, not a service.
const BUILT_IN: &[&str] = &["builtin", "library"];
/// Music Assistant's default provider names that are too long to show.
const SHORT_NAMES: &[(&str, &str)] = &[("OpenSubsonic Media Server Library", "Subsonic")];

/// What one poll of Music Assistant returned.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Responses {
    pub player: api::Player,
    pub queue: api::Queue,
    /// The queue entries after the current one.
    pub upcoming: Vec<api::QueueItem>,
    pub providers: Vec<api::ProviderInfo>,
}

/// The content Music Assistant reports for `player_key`, or `None` when it
/// reports nothing playing.
pub fn to_content(
    responses: &Responses,
    base_url: &str,
    player_key: &str,
    now: Timestamp,
) -> Option<Content> {
    let media = responses.player.current_media.as_ref();
    let item = responses.queue.current_item.as_ref();
    let track = track(item, media)?;
    let details = item.and_then(|item| item.streamdetails.as_ref());

    Some(Content {
        track,
        artwork: artwork(item, media, base_url),
        position: position(&responses.queue, item, media, now),
        quality: details.and_then(|details| quality(details, player_key)),
        delivery: details.map(|details| delivery(details, player_key)),
        queue: QueueView::Available(
            responses
                .upcoming
                .iter()
                .take(UP_NEXT)
                .map(|entry| queue_entry(entry, base_url))
                .collect(),
        ),
        service: details
            .and_then(|details| details.provider.as_deref())
            .and_then(|instance| service(instance, &responses.providers)),
        palette: media
            .and_then(|media| media.palette.as_ref())
            .and_then(palette),
    })
}

/// The music provider instance of the current item.
pub fn provider_instance(responses: &Responses) -> Option<&str> {
    responses
        .queue
        .current_item
        .as_ref()?
        .streamdetails
        .as_ref()?
        .provider
        .as_deref()
}

/// Whether the provider list names `instance` (or it needs no name).
pub fn knows_provider(instance: &str, providers: &[api::ProviderInfo]) -> bool {
    BUILT_IN.contains(&instance) || providers.iter().any(|p| p.instance_id == instance)
}

fn track(item: Option<&api::QueueItem>, media: Option<&api::CurrentMedia>) -> Option<Track> {
    if let Some(media_item) = item.and_then(|item| item.media_item.as_ref()) {
        return Some(Track {
            title: media_item.name.clone(),
            artists: media_item
                .artists
                .iter()
                .map(|artist| artist.name.clone())
                .collect(),
            album: media_item.album.as_ref().map(|album| album.name.clone()),
        });
    }
    let media = media?;
    Some(Track {
        title: media.title.clone()?,
        artists: media.artist.iter().cloned().collect(),
        album: media.album.clone(),
    })
}

fn artwork(
    item: Option<&api::QueueItem>,
    media: Option<&api::CurrentMedia>,
    base_url: &str,
) -> Option<ArtworkRef> {
    item.and_then(image)
        .and_then(|image| image_url(image, base_url, 0))
        .or_else(|| media.and_then(|media| media.image_url.clone()))
        .map(ArtworkRef::new)
}

/// The item's own image, else its album's, else the first in its metadata.
fn image(item: &api::QueueItem) -> Option<&api::Image> {
    let media_item = item.media_item.as_ref();
    item.image
        .as_ref()
        .or_else(|| media_item?.album.as_ref()?.image.as_ref())
        .or_else(|| media_item?.metadata.as_ref()?.images.as_ref()?.first())
}

/// The image's own URL when it is remotely accessible over http(s),
/// otherwise Music Assistant's image proxy at `size` (0 for the original).
fn image_url(image: &api::Image, base_url: &str, size: u32) -> Option<String> {
    let remote = image.remotely_accessible
        && (image.path.starts_with("http://") || image.path.starts_with("https://"));
    if remote && size == 0 {
        return Some(image.path.clone());
    }
    match &image.proxy_id {
        Some(proxy_id) => Some(proxy(base_url, proxy_id, size)),
        None if remote => Some(image.path.clone()),
        None => None,
    }
}

fn proxy(base_url: &str, proxy_id: &str, size: u32) -> String {
    format!("{base_url}/imageproxy/{proxy_id}?size={size}&fmt=jpg")
}

fn position(
    queue: &api::Queue,
    item: Option<&api::QueueItem>,
    media: Option<&api::CurrentMedia>,
    now: Timestamp,
) -> Option<Position> {
    // The queue's elapsed time is per track; the player's counts across the
    // whole flow stream and is never used.
    let elapsed = queue.elapsed_time?;
    let duration = item
        .and_then(|item| item.duration.or_else(|| item.media_item.as_ref()?.duration))
        .or_else(|| media?.duration)
        .and_then(seconds);
    let observed_at = queue
        .elapsed_time_last_updated
        .and_then(timestamp)
        .filter(|measured| distance(*measured, now) <= MAX_CLOCK_SKEW)
        .unwrap_or(now);
    Some(Position {
        elapsed: seconds(elapsed)?,
        duration,
        observed_at,
    })
}

fn distance(a: Timestamp, b: Timestamp) -> Duration {
    a.since(b).max(b.since(a))
}

fn seconds(value: f64) -> Option<Duration> {
    Duration::try_from_secs_f64(value).ok()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn timestamp(unix_seconds: f64) -> Option<Timestamp> {
    (unix_seconds.is_finite() && unix_seconds >= 0.0)
        .then(|| Timestamp::from_millis((unix_seconds * 1000.0).round() as u64))
}

fn quality(details: &api::StreamDetails, player_key: &str) -> Option<AudioQuality> {
    let source = details.audio_format.as_ref()?;
    let quality = measured(source);

    let mut detail = Vec::new();
    if quality.is_lossless() && !quality.is_hi_res() {
        detail.push("Lossless".to_owned());
    }
    if let Some(output) = output_format(details, player_key)
        && (output.bit_depth, output.sample_rate) != (source.bit_depth, source.sample_rate)
    {
        // Without a codec the label is just " <bits>/<kHz>".
        let converted = AudioQuality::format(AudioFormat {
            codec: String::new(),
            bit_depth: output.bit_depth,
            sample_rate_hz: output.sample_rate,
        });
        detail.push(format!("→ {} to player", converted.label().trim()));
    }
    if normalises(details) {
        detail.push("Normalised".to_owned());
    }

    Some(if detail.is_empty() {
        quality
    } else {
        quality.with_detail(detail.join(" · "))
    })
}

/// A format as a quality: the bitrate for a lossy codec that reports one,
/// otherwise the format.
fn measured(source: &api::AudioFormat) -> AudioQuality {
    let format = AudioFormat {
        codec: source.content_type.clone(),
        bit_depth: source.bit_depth,
        sample_rate_hz: source.sample_rate,
    };
    let lossless = AudioQuality::format(format.clone()).is_lossless();
    if !lossless && source.bit_rate > 0 {
        AudioQuality::bitrate(source.bit_rate * 1000)
    } else {
        AudioQuality::format(format)
    }
}

/// What Music Assistant sends to this player, and what it changed on the
/// way.
fn delivery(details: &api::StreamDetails, player_key: &str) -> Delivery {
    let source = details.audio_format.as_ref();
    let output = output(details, player_key);
    let format = output.and_then(|output| output.output_format.as_ref());

    let mut changes = Vec::new();
    if let (Some(source), Some(format)) = (source, format) {
        if format.sample_rate != source.sample_rate {
            changes.push(Change::Resampled);
        } else if format.bit_depth != source.bit_depth {
            changes.push(Change::BitDepth);
        }
    }
    if normalises(details) {
        let gain = details
            .audio_processing
            .as_ref()
            .and_then(|processing| processing.queue_processing.as_ref())
            .and_then(|queue| queue.normalization.as_ref())
            .and_then(|normalization| normalization.applied_gain_db)
            .or(details.volume_normalization_gain_correct)
            .filter(|db| db.is_finite())
            .map(Decibels::from_db);
        changes.push(Change::Normalised { gain });
    }
    if output
        .and_then(|output| output.dsp.as_ref())
        .is_some_and(|dsp| dsp.state == "enabled")
    {
        changes.push(Change::Dsp);
    }

    Delivery {
        quality: format.map(measured),
        changes,
    }
}

fn normalises(details: &api::StreamDetails) -> bool {
    details
        .volume_normalization_mode
        .as_deref()
        .is_some_and(|mode| ALTERS_VOLUME.contains(&mode))
}

/// The output that serves this player (the only one when none names it).
fn output<'a>(details: &'a api::StreamDetails, player_key: &str) -> Option<&'a api::Output> {
    let outputs = &details.audio_processing.as_ref()?.outputs;
    outputs
        .iter()
        .find(|output| output.player_ids.iter().any(|id| id == player_key))
        .or_else(|| (outputs.len() == 1).then(|| &outputs[0]))
}

/// The format Music Assistant sends to this player.
fn output_format<'a>(
    details: &'a api::StreamDetails,
    player_key: &str,
) -> Option<&'a api::AudioFormat> {
    output(details, player_key)?.output_format.as_ref()
}

fn queue_entry(entry: &api::QueueItem, base_url: &str) -> QueueEntry {
    let media_item = entry.media_item.as_ref();
    QueueEntry {
        title: media_item.map_or_else(|| entry.name.clone(), |item| item.name.clone()),
        artists: media_item
            .map(|item| item.artists.iter().map(|a| a.name.clone()).collect())
            .unwrap_or_default(),
        artwork: image(entry)
            .and_then(|image| image_url(image, base_url, 256))
            .map(ArtworkRef::new),
    }
}

/// The music service of the provider `instance`: named as Music Assistant
/// names it, branded by the provider's domain, which renaming doesn't change.
fn service(instance: &str, providers: &[api::ProviderInfo]) -> Option<Service> {
    if BUILT_IN.contains(&instance) {
        return None;
    }
    let provider = providers
        .iter()
        .find(|provider| provider.instance_id == instance)?;
    let name = &provider.name;
    let short = SHORT_NAMES
        .iter()
        .find(|(long, _)| long == name)
        .map_or(name.as_str(), |(_, short)| short);
    (!short.is_empty()).then(|| Service::new(brand(provider), short))
}

/// The brand of a provider, from its domain. Music Assistant doesn't say
/// which Subsonic server is behind `opensubsonic`: Navidrome when the user
/// named the provider so, Subsonic otherwise.
fn brand(provider: &api::ProviderInfo) -> Option<Brand> {
    let id = match provider.domain.as_str() {
        "opensubsonic" if provider.name.to_ascii_lowercase().contains("navidrome") => "navidrome",
        "opensubsonic" => "subsonic",
        domain if domain.starts_with("filesystem_") => "library",
        "qobuz" => "qobuz",
        "spotify" => "spotify",
        "tidal" => "tidal",
        "deezer" => "deezer",
        "apple_music" => "apple-music",
        "plex" => "plex",
        "jellyfin" => "jellyfin",
        "soundcloud" => "soundcloud",
        "ytmusic" => "youtube-music",
        _ => return None,
    };
    Brand::new(id)
}

fn palette(palette: &api::Palette) -> Option<Palette> {
    let [r, g, b] = palette.background_dark?;
    let [ar, ag, ab] = palette.on_dark?;
    Some(Palette {
        background: Rgb(r, g, b),
        accent: Rgb(ar, ag, ab),
    })
}
