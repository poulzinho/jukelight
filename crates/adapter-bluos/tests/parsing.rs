//! Spec: device-bluos, tested against recorded responses (example values).

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::fs;
use std::path::Path;
use std::time::Duration;

use jukelight_adapter_bluos::parse::{Identity, parse_playlist, parse_status, parse_sync_status};
use jukelight_adapter_bluos::to_device_snapshot;
use jukelight_domain::{
    AudioQuality, DeviceSnapshot, Origin, PlaybackState, QualityMeasure, QueueView, Timestamp,
};

const BASE: &str = "http://192.168.1.40:11000";
const NOW: Timestamp = Timestamp::from_millis(1_000_000);

fn fixture(name: &str, file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .join(file);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn identity() -> Identity {
    parse_sync_status(&fixture("qobuz-native-playing", "syncstatus.xml")).unwrap()
}

/// The device snapshot for a recorded situation.
fn snapshot(name: &str) -> DeviceSnapshot {
    let status = parse_status(&fixture(name, "status.xml")).unwrap();
    let playlist = parse_playlist(&fixture(name, "playlist.xml")).unwrap();
    to_device_snapshot(&status, &identity(), &playlist, BASE, NOW)
}

fn inline_status(fields: &str) -> DeviceSnapshot {
    let xml = format!(r#"<?xml version="1.0"?><status etag="e1">{fields}</status>"#);
    to_device_snapshot(&parse_status(&xml).unwrap(), &identity(), &[], BASE, NOW)
}

// Player identity

#[test]
fn identity_from_sync_status() {
    let identity = identity();
    assert_eq!(identity.name, "Example Player");
    assert_eq!(identity.player_key, "aabbccddeeff");
}

// Playback state mapping

#[test]
fn cast_session_reports_stream_as_playing() {
    assert_eq!(
        snapshot("qobuz-connect-playing").state,
        PlaybackState::Playing
    );
}

#[test]
fn native_play_is_playing() {
    assert_eq!(
        snapshot("qobuz-native-playing").state,
        PlaybackState::Playing
    );
}

#[test]
fn paused() {
    for name in [
        "qobuz-connect-paused",
        "spotify-connect-paused",
        "ma-stream-paused",
    ] {
        assert_eq!(snapshot(name).state, PlaybackState::Paused, "{name}");
    }
}

#[test]
fn connecting_is_buffering() {
    assert_eq!(
        snapshot("spotify-connect-connecting").state,
        PlaybackState::Buffering
    );
}

#[test]
fn stopped() {
    assert_eq!(
        snapshot("qobuz-native-stopped").state,
        PlaybackState::Stopped
    );
}

// Origin

#[test]
fn music_assistant_stream() {
    assert_eq!(
        snapshot("ma-stream-playing").origin,
        Origin::Stream {
            url: "http://192.168.1.50:8097/flow/session1/aabbccddeeff/00000000000000000000000000000001/aabbccddeeff.flac".into()
        }
    );
}

#[test]
fn cast_service() {
    assert_eq!(
        snapshot("spotify-connect-playing").origin,
        Origin::Service {
            id: "Spotify".into(),
            name: "Spotify".into(),
            cast: true
        }
    );
    assert_eq!(
        snapshot("qobuz-connect-playing").origin,
        Origin::Service {
            id: "QobuzConnect".into(),
            name: "Qobuz Connect".into(),
            cast: true
        }
    );
}

#[test]
fn native_service() {
    assert_eq!(
        snapshot("qobuz-native-playing").origin,
        Origin::Service {
            id: "Qobuz".into(),
            name: "Qobuz".into(),
            cast: false
        }
    );
}

// The player's own content

#[test]
fn full_metadata() {
    let content = snapshot("qobuz-native-playing").content.unwrap();
    assert_eq!(content.track.title, "Cement Head");
    assert_eq!(content.track.artists, vec!["Caroline Rose".to_owned()]);
    assert_eq!(
        content.track.album.as_deref(),
        Some("Gentling the Horse at the Walt Whitman Mall")
    );
    assert_eq!(
        content.position.unwrap().duration,
        Some(Duration::from_secs(131))
    );
    let quality = content.quality.unwrap();
    assert_eq!(quality.label(), "FLAC 24/96");
    assert!(quality.is_hi_res());
}

#[test]
fn title_lines_as_fallback() {
    let content = inline_status(
        "<state>play</state><service>Qobuz</service><title1>Song</title1><title2>Artist</title2><title3>Album</title3>",
    )
    .content
    .unwrap();
    assert_eq!(content.track.artists, vec!["Artist".to_owned()]);
    assert_eq!(content.track.album.as_deref(), Some("Album"));
}

#[test]
fn escaped_characters() {
    let content = inline_status("<state>play</state><title1>Me &amp; You</title1>")
        .content
        .unwrap();
    assert_eq!(content.track.title, "Me & You");
}

#[test]
fn relative_artwork_path() {
    let content = snapshot("qobuz-native-playing").content.unwrap();
    assert_eq!(
        content.artwork.unwrap().source(),
        "http://192.168.1.40:11000/Artwork?service=Qobuz&songid=Qobuz%3A432362518"
    );
}

#[test]
fn absolute_artwork_url() {
    let content = snapshot("spotify-connect-playing").content.unwrap();
    assert_eq!(
        content.artwork.unwrap().source(),
        "https://i.scdn.co/image/ab67616d0000b273dedbe9d8db9f64e075ae1764"
    );
}

#[test]
fn bitrate_instead_of_format() {
    let quality = snapshot("spotify-connect-playing")
        .content
        .unwrap()
        .quality
        .unwrap();
    assert_eq!(
        quality.measure,
        QualityMeasure::Bitrate {
            bits_per_second: 160_000
        }
    );
    assert_eq!(quality.label(), "160 kbps");
}

#[test]
fn cd_quality_detail() {
    let quality: AudioQuality = snapshot("qobuz-connect-paused")
        .content
        .unwrap()
        .quality
        .unwrap();
    assert_eq!(quality.label(), "FLAC 16/44.1");
    assert_eq!(quality.detail.as_deref(), Some("CD quality"));
}

#[test]
fn zero_quality_is_no_quality() {
    assert_eq!(
        snapshot("qobuz-native-stopped").content.unwrap().quality,
        None
    );
}

#[test]
fn no_title_means_no_content() {
    assert_eq!(inline_status("<state>stop</state>").content, None);
}

#[test]
fn cast_session_starting_shows_the_service_as_title() {
    // Recorded at the moment Qobuz Connect started, before metadata arrived.
    let content = snapshot("qobuz-connect-playing").content.unwrap();
    assert_eq!(content.track.title, "Qobuz Connect");
    assert_eq!(content.quality, None);
}

// Position from the player

#[test]
fn native_session_reports_position() {
    let position = snapshot("qobuz-native-playing")
        .content
        .unwrap()
        .position
        .unwrap();
    assert_eq!(position.elapsed, Duration::from_secs(109));
    assert_eq!(position.observed_at, NOW);
}

#[test]
fn stream_origin_reports_no_position() {
    assert_eq!(
        snapshot("ma-stream-playing").content.unwrap().position,
        None
    );
}

// Quality only for service sessions

#[test]
fn converted_stream_reports_no_quality() {
    // Music Assistant streams FLAC 24/44.1 even for 16-bit sources.
    for name in ["ma-stream-playing", "ma-stream-paused"] {
        assert_eq!(snapshot(name).content.unwrap().quality, None, "{name}");
    }
}

// Native queue (task 4.4)

#[test]
fn native_queue() {
    let content = snapshot("qobuz-native-playing").content.unwrap();
    let QueueView::Available(entries) = content.queue else {
        panic!("queue should be available")
    };
    // `song` is 3 in a playlist of 11: the upcoming entries are 5 to 11.
    assert_eq!(entries.len(), 7);
    let first = &entries[0];
    assert_eq!(first.artists, vec!["Caroline Rose".to_owned()]);
    assert!(first.artwork.as_ref().unwrap().source().starts_with(BASE));
}

#[test]
fn stale_playlist_during_a_cast_session() {
    for name in ["spotify-connect-playing", "qobuz-connect-playing"] {
        let content = snapshot(name).content.unwrap();
        assert_eq!(content.queue, QueueView::NotApplicable, "{name}");
    }
}

#[test]
fn stream_has_no_queue() {
    assert_eq!(
        snapshot("ma-stream-playing").content.unwrap().queue,
        QueueView::NotApplicable
    );
}

#[test]
fn playlist_entries_are_decoded() {
    let entries = parse_playlist(&fixture("qobuz-native-playing", "playlist.xml")).unwrap();
    assert_eq!(entries.len(), 11);
    assert_eq!(entries[0].title, "Yip Yip Yow");
    assert_eq!(entries[0].artist.as_deref(), Some("Caroline Rose"));
    assert_eq!(
        entries[0].image.as_deref(),
        Some("/Artwork?service=Qobuz&songid=Qobuz%3A432362515")
    );
}

#[test]
fn status_etag() {
    let status = parse_status(&fixture("qobuz-native-playing", "status.xml")).unwrap();
    assert!(!status.etag.is_empty());
    assert_eq!(status.get("pid"), Some("426"));
}
