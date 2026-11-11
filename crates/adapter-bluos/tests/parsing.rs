//! Spec: device-bluos, tested against recorded responses (example values).

#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on bad test data

use std::fs;
use std::path::Path;
use std::time::Duration;

use jukelight_adapter_bluos::parse::{Identity, parse_playlist, parse_status, parse_sync_status};
use jukelight_adapter_bluos::to_device_snapshot;
use jukelight_domain::{
    AudioInput, AudioQuality, Brand, DeviceSnapshot, InputKind, Origin, Output, OutputKind,
    PlaybackState, QualityMeasure, QueueView, Timestamp,
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

// Spec: device-bluos, "Player maker".

#[test]
fn bluesound_player() {
    assert_eq!(identity().maker, Brand::new("bluesound"));
    assert_eq!(
        snapshot("qobuz-native-playing").maker,
        Brand::new("bluesound")
    );
}

#[test]
fn another_bluos_maker() {
    let xml = fixture("qobuz-native-playing", "syncstatus.xml")
        .replace(r#"brand="Bluesound""#, r#"brand="Example Maker""#);
    assert_eq!(parse_sync_status(&xml).unwrap().maker, None);
}

// Spec: device-bluos, "Service brand".

#[test]
fn unknown_service() {
    let snapshot = inline_status(
        "<service>ExampleService</service><serviceName>Example Service</serviceName><state>play</state>",
    );
    let Origin::Service { brand, name, .. } = snapshot.origin else {
        panic!("not a service: {:?}", snapshot.origin)
    };
    assert_eq!(brand, None);
    assert_eq!(name, "Example Service");
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
            brand: Brand::new("spotify"),
            id: "Spotify".into(),
            name: "Spotify".into(),
            cast: true
        }
    );
    assert_eq!(
        snapshot("qobuz-connect-playing").origin,
        Origin::Service {
            brand: Brand::new("qobuz"),
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
            brand: Brand::new("qobuz"),
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

// Audio inputs (recorded on the reference player with a CD transport on the
// optical input; the input's name is an example)

fn input_of(snapshot: &DeviceSnapshot) -> &AudioInput {
    match &snapshot.origin {
        Origin::Input(input) => input,
        other => panic!("expected an input origin, got {other:?}"),
    }
}

#[test]
fn optical_input() {
    let snapshot = snapshot("optical-input-playing");
    let input = input_of(&snapshot);
    assert_eq!(input.kind, InputKind::Optical);
    assert_eq!(input.name, "Example Input");
    assert_eq!(snapshot.state, PlaybackState::Playing);
    // Not a track: no title, no queue, even though /Playlist still holds
    // an earlier session's entries.
    assert_eq!(snapshot.content, None);
}

#[test]
fn named_optical_input() {
    let snapshot = inline_status(
        "<service>Capture</service><serviceType>AudioInputs</serviceType>\
         <inputTypeIndex>spdif-1</inputTypeIndex><title1>Leak</title1>\
         <state>stream</state><secs>314</secs>",
    );
    let input = input_of(&snapshot);
    assert_eq!(input.name, "Leak");
    assert_eq!(
        input.started_at,
        Some(Timestamp::from_millis(NOW.as_millis() - 314_000))
    );
    assert_eq!(snapshot.content, None);
}

#[test]
fn waiting_for_a_signal() {
    let snapshot = snapshot("optical-input-connecting");
    let input = input_of(&snapshot);
    assert_eq!(input.started_at, None);
    assert_eq!(snapshot.state, PlaybackState::Buffering);
}

#[test]
fn streaming_input_has_a_start_time() {
    let snapshot = snapshot("optical-input-playing");
    // Recorded right when the signal started (`secs` 0).
    assert_eq!(input_of(&snapshot).started_at, Some(NOW));
}

#[test]
fn bluetooth_input() {
    let snapshot = inline_status(
        "<service>Capture</service><serviceType>AudioInputs</serviceType>\
         <inputTypeIndex>bluetooth-1</inputTypeIndex><title1>Bluetooth</title1>\
         <state>stream</state>",
    );
    assert!(
        matches!(snapshot.origin, Origin::Service { .. }),
        "{:?}",
        snapshot.origin
    );
}

#[test]
fn input_kinds() {
    for (index, kind) in [
        ("spdif-1", InputKind::Optical),
        ("analog-1", InputKind::Analog),
        ("arc-1", InputKind::HdmiArc),
        ("hdmi-1", InputKind::HdmiArc),
        ("usb-1", InputKind::Other),
    ] {
        let snapshot = inline_status(&format!(
            "<serviceType>AudioInputs</serviceType><inputTypeIndex>{index}</inputTypeIndex>\
             <title1>In</title1><state>stream</state><secs>1</secs>"
        ));
        assert_eq!(input_of(&snapshot).kind, kind, "{index}");
    }
}

#[test]
fn unnamed_input() {
    let snapshot = inline_status(
        "<serviceType>AudioInputs</serviceType><inputTypeIndex>analog-1</inputTypeIndex>\
         <state>stream</state><secs>5</secs>",
    );
    assert_eq!(input_of(&snapshot).name, "Input");
}

// Spec: device-bluos, "Subwoofer".

/// A snapshot from one situation's own `/Status` and `/SyncStatus`.
fn snapshot_with_identity(name: &str) -> DeviceSnapshot {
    let status = parse_status(&fixture(name, "status.xml")).unwrap();
    let identity = parse_sync_status(&fixture(name, "syncstatus.xml")).unwrap();
    to_device_snapshot(&status, &identity, &[], BASE, NOW)
}

#[test]
fn subwoofer_on() {
    assert_eq!(
        snapshot_with_identity("sub-on").outputs,
        vec![Output {
            name: None,
            kind: OutputKind::Subwoofer
        }]
    );
}

#[test]
fn subwoofer_off() {
    assert_eq!(snapshot_with_identity("sub-off").outputs, vec![]);
}
